use lazy_static::lazy_static;
use pyo3::prelude::*;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BigbotError {
    #[error("Python error: {0}")]
    PythonError(#[from] PyErr),
    #[error("System error: {0}")]
    SystemError(String),
}

lazy_static! {
    pub static ref SPACY: SpacyModule = {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let spacy = py.import("spacy").unwrap();
        py.import("spacytextblob.spacytextblob").unwrap();
        let mut models = HashMap::new();
        let langs = vec!["en_core_web_lg"];
        for lang in langs {
            let en_model = spacy.call(py, "load", (lang,), None).unwrap();
            en_model
                .call_method(py, "add_pipe", ("spacytextblob",), None)
                .unwrap();
            models.insert(
                lang,
                LangModel {
                    po: Arc::new(en_model),
                },
            );
        }
        SpacyModule {
            default_lang: "en_core_web_lg",
            models: models,
        }
    };
}

#[derive(Debug)]
pub struct SpacyModule {
    pub default_lang: &'static str,
    pub models: HashMap<&'static str, LangModel>,
}

impl SpacyModule {
    pub fn model_default(&self) -> &LangModel {
        self.models.get(self.default_lang).unwrap()
    }

    pub fn model_in_lang(&self, lang: &str) -> Option<&LangModel> {
        self.models.get(lang)
    }
}

#[derive(Debug, Clone)]
pub struct LangModel {
    po: Arc<PyObject>,
}

impl LangModel {
    pub async fn nlp(&self, text: String) -> Result<Doc, BigbotError> {
        let model = self.po.clone();
        tokio::task::spawn_blocking(move || -> Result<Doc, BigbotError> {
            let gil = Python::acquire_gil();
            let py = gil.python();
            let result = model.call(py, (text,), None)?;
            Ok(Doc::new(result))
        })
        .await
        .map_err(|x| BigbotError::SystemError(format!("{}", x)))?
    }
}

#[derive(Debug)]
pub struct Doc {
    pub object: PyObject,
}

impl Doc {
    pub fn new(object: PyObject) -> Self {
        Doc { object: object }
    }

    pub fn ents(&self, py: Python) -> Result<Vec<SpacyEntity>, BigbotError> {
        let ents: Vec<PyObject> = self.object.getattr(py, "ents")?.extract(py)?;
        Ok(ents.into_iter().map(|obj| SpacyEntity { obj }).collect())
    }

    pub fn sentiment(&self, py: Python) -> Result<f64, BigbotError> {
        Ok(self
            .object
            .getattr(py, "_")?
            .getattr(py, "blob")?
            .getattr(py, "polarity")?
            .extract::<f64>(py)?)
    }

    pub fn tokens(&self, py: Python) -> Result<Vec<Token>, BigbotError> {
        let token_objs: Vec<PyObject> = self.object.extract(py)?;
        let mut tokens = Vec::with_capacity(token_objs.len());
        for obj in token_objs {
            tokens.push(Token::try_from((py, obj))?);
        }
        Ok(tokens)
    }

    pub fn named_entities(&self, py: Python) -> Result<Vec<(String, usize, usize)>, BigbotError> {
        let ents = self.ents(py)?;
        let mut named_entities = Vec::new();
        for ent in ents {
            let text = ent.text(py)?;
            let start = ent.start_char(py)?;
            let end = ent.end_char(py)?;
            named_entities.push((text, start, end));
        }
        Ok(named_entities)
    }

    pub fn lemmas(&self, py: Python) -> Result<Vec<String>, BigbotError> {
        let tokens = self.tokens(py)?;
        let lemmas: Result<Vec<String>, BigbotError> = tokens
            .into_iter()
            .map(|token| token.lemma(py))
            .collect();
        lemmas
    }
}

#[derive(Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Copy)]
pub enum EntityLabel {
    Person,
    Org,
    Gpe,
    Loc,
    Date,
    Time,
    Money,
    Norp,
    WorkOfArt,
    Phone,
    Email,
    Cardinal,
    Unknown,
}

impl ToString for EntityLabel {
    fn to_string(&self) -> String {
        match self {
            EntityLabel::Person => "PERSON",
            EntityLabel::Org => "ORG",
            EntityLabel::Gpe => "GPE",
            EntityLabel::Loc => "LOC",
            EntityLabel::Date => "DATE",
            EntityLabel::Time => "TIME",
            EntityLabel::Money => "MONEY",
            EntityLabel::Norp => "NORP",
            EntityLabel::WorkOfArt => "WORK_OF_ART",
            EntityLabel::Phone => "PHONE",
            EntityLabel::Email => "EMAIL",
            EntityLabel::Cardinal => "CARDINAL",
            EntityLabel::Unknown => "",
        }
        .to_string()
    }
}

impl FromStr for EntityLabel {
    type Err = BigbotError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PERSON" => Ok(EntityLabel::Person),
            "ORG" => Ok(EntityLabel::Organization),
            "GPE" => Ok(EntityLabel::GeopoliticalEntity),
            "LOC" => Ok(EntityLabel::Location),
            "PRODUCT" => Ok(EntityLabel::Product),
            "EVENT" => Ok(EntityLabel::Event),
            "WORK_OF_ART" => Ok(EntityLabel::WorkOfArt),
            "LAW" => Ok(EntityLabel::Law),
            "LANGUAGE" => Ok(EntityLabel::Language),
            "DATE" => Ok(EntityLabel::Date),
            "TIME" => Ok(EntityLabel::Time),
            "PERCENT" => Ok(EntityLabel::Percent),
            "MONEY" => Ok(EntityLabel::Money),
            "QUANTITY" => Ok(EntityLabel::Quantity),
            "ORDINAL" => Ok(EntityLabel::Ordinal),
            "CARDINAL" => Ok(EntityLabel::Cardinal),
            _ => Err(BigbotError::EntityLabelConversionError(format!("Invalid entity label: {}", s))),
        }
    }
}

#[pyclass]
#[derive(Clone, Debug, Ord, PartialOrd, Eq, PartialEq, Hash)]
pub struct Entity {
    pub text: String,
    pub label: EntityLabel,
}

#[derive(Default, Debug)]
pub struct EntityGraph {
    entities: HashMap<EntityLabel, HashSet<String>>,
}

impl EntityGraph {
    pub fn add_entity(&mut self, label: EntityLabel, text: String) {
        let texts = self.entities.entry(label).or_insert(HashSet::new());
        texts.insert(text);
    }
}

#[pyclass]
#[derive(Clone)]
pub struct SpacyEntity {
    obj: PyObject,
}

#[pymethods]
impl SpacyEntity {
    pub fn export(&self, py: Python) -> Result<Entity, BigbotError> {
        let text: String = self.obj.getattr(py, "text")?.extract(py)?;
        let label: EntityLabel = self
            .obj
            .getattr(py, "label_")?
            .extract::<String>(py)?
            .into();
        Ok(Entity { text, label })
    }

    pub fn start_char(&self, py: Python) -> Result<usize, BigbotError> {
        Ok(self.obj.getattr(py, "start_char")?.extract(py)?)
    }

    pub fn end_char(&self, py: Python) -> Result<usize, BigbotError> {
        Ok(self.obj.getattr(py, "end_char")?.extract(py)?)
    }

    pub fn text(&self, py: Python) -> Result<String, BigbotError> {
        Ok(self.obj.getattr(py, "text")?.extract(py)?)
    }
}

impl ToPyObject for SpacyEntity {
    fn to_object(&self, py: Python<'_>) -> PyObject {
        self.clone().into_py(py)
    }
}

#[derive(Debug)]
pub struct Token {
    obj: PyObject,
    pub pos: TokenPos,
    pub text: String,
    pub entity_type: String,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TokenPos {
    NOUN,
    ADJ,
    PUNCT,
    PROPN,
    DOBJ,
    VERB,
    OTHERS,
}

impl From<String> for TokenPos {
    fn from(value: String) -> Self {
        match value.to_ascii_uppercase().as_str() {
            "NOUN" => TokenPos::NOUN,
            "ADJ" => TokenPos::ADJ,
            "PUNCT" => TokenPos::PUNCT,
            "PROPN" => TokenPos::PROPN,
            "DOBJ" => TokenPos::DOBJ,
            "VERB" => TokenPos::VERB,
            _ => TokenPos::OTHERS,
        }
    }
}

impl TryFrom<(Python<'_>, PyObject)> for Token {
    type Error = BigbotError;

    fn try_from((py, obj): (Python, PyObject)) -> Result<Self, Self::Error> {
        Ok(Token {
            text: obj.getattr(py, "text")?.extract(py)?,
            pos: obj
                .getattr(py, "pos_")?
                .extract::<String>(py)?
                .into(),
            entity_type: obj.getattr(py, "ent_type_")?.extract(py)?,
            obj: obj,
        })
    }
}

impl Token {
    pub fn children(&self, py: Python) -> Result<Vec<Token>, BigbotError> {
        let children_generator = self.obj.getattr(py, "children")?;
        let mut children = Vec::new();
        while let Some(child) = children_generator.iter(py)?.next() {
            let token = Token::try_from((py, child?))?;
            children.push(token);
        }
        Ok(children)
    }

    pub fn dep(&self, py: Python) -> Result<String, BigbotError> {
        Ok(self.obj.getattr(py, "dep_")?.extract(py)?)
    }

    pub fn head(&self, py: Python) -> Result<Token, BigbotError> {
        Token::try_from((py, self.obj.getattr(py, "head")?))
    }

    pub fn lemma(&self, py: Python) -> Result<String, BigbotError> {
        Ok(self.obj.getattr(py, "lemma_")?.extract(py)?)
    }
}