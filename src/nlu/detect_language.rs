// Use rust_cpython to interface with Python for language detection using Spacy and Rasa
use rust_cpython::{PyResult, Python};

/// Detects languages of the given utterance using Spacy and Rasa libraries.
///
/// # Arguments
/// * `utterance` - A string slice containing the text for language detection.
///
/// # Returns
/// * A PyResult wrapping a vector of detected languages as strings.
///
/// # Examples
/// ```
/// let detected_languages = detect_languages("Bonjour, comment allez-vous?").unwrap();
/// println!("{:?}", detected_languages); // Outputs: ["fra"]
/// ```

use pyo3::prelude::*;
use pyo3::types::PyDict;

pub fn detect_languages(utterance: &str) -> PyResult<Vec<String>> {
    // Acquire the Global Interpreter Lock (GIL) and get Python interpreter instance
    let gil = Python::acquire_gil();
    let py = gil.python();

    // Import and initialize Spacy for language processing
    let spacy = py.import("spacy")?;
    let nlp = spacy.call(py, "load", ("en_core_web_sm",), None)?;

    // Import and initialize Rasa for language detection
    let rasa = py.import("rasa")?;
    let language_detector = rasa.call(py, "language.LanguageDetector", (), None)?;

    // Process the utterance using Spacy to create a document object
    let doc = nlp.call(py, "make_doc", (utterance,), None)?;

    // Temporarily disable named entity recognition for efficiency
    let processed_doc = nlp.call_method(py, "disable_pipes", ("ner",))?;
    processed_doc.call_method(py, "set_ents", (doc.to_object(py),))?;

    // Detect language(s) using the Rasa language detector on the processed document
    let language_result = language_detector.call_method(py, "detect_language", (processed_doc.to_object(py),))?;

    // Extract and return the detected languages
    let detected_languages: Vec<String> = language_result.extract(py)?;

    // Novel implementation: Detect language confidence scores
    let confidence_scores = language_detector.call_method(py, "get_confidence_scores", (processed_doc.to_object(py),))?;
    let confidence_scores_dict: &PyDict = confidence_scores.extract(py)?;
    for (lang, score) in confidence_scores_dict {
        println!("Language: {}, Confidence Score: {}", lang, score);
    }

    Ok(detected_languages)
}

fn main() {
    // Example usage of the detect_languages function
    let detected_languages = detect_languages("Bonjour, comment allez-vous?").unwrap();
    println!("Detected Languages: {:?}", detected_languages);
}