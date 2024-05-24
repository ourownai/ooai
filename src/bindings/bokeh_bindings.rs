use pyo3::prelude::*;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::collections::{HashSet, HashMap};
use std::iter::FromIterator;

// Define the DataBin struct
pub struct DataBin {
    pub data: Vec<HashMap<String, String>>,
    pub fields: Vec<String>,
}

impl DataBin {
    pub fn new(data: Vec<HashMap<String, String>>, fields: Vec<String>) -> Self {
        DataBin { data, fields }
    }
}

// Define the ChartConfig struct
pub struct ChartConfig {
    plot_width: u32,
    plot_height: u32,
}

impl ChartConfig {
    pub fn new() -> Self {
        ChartConfig {
            plot_width: 800,
            plot_height: 600,
        }
    }

    pub fn set_plot_width(&mut self, width: u32) {
        self.plot_width = width;
    }

    pub fn set_plot_height(&mut self, height: u32) {
        self.plot_height = height;
    }
}

// Define the Rust function
#[pyfunction]
fn my_rust_function() -> PyResult<String> {
    Ok("Hello, world!".to_string())
}

// Define the Python module
#[pymodule]
fn myrustlib(_py: Python, m: &PyModule) -> PyResult<()> {
    // Add the Rust function to the module
    m.add_function(wrap_pyfunction!(my_rust_function, m)?)?;
    Ok(())
}

// Create a global static variable for the Bokeh Python module, wrapped in a Mutex for thread safety
lazy_static! {
    pub static ref BOKEH: Mutex<Py<PyAny>> = {
        // Acquire the Global Interpreter Lock (GIL) and get the Python interpreter
        let gil = Python::acquire_gil();
        let py = gil.python();
        // Import the Bokeh module and return it wrapped in a Mutex
        let bokeh = py.import("bokeh").unwrap();
        Mutex::new(bokeh.into())
    };
}

// Function to prepare data for the chart
pub fn prepare_data_for_chart(data_bin: &DataBin, chart_type: &str) -> HashMap<String, Vec<String>> {
    let mut chart_data: HashMap<String, Vec<String>> = HashMap::new();

    // Prepare data based on the chart type
    match chart_type {
        "line" | "bar" => {
            let x_data: Vec<String> = data_bin.data.iter().map(|item| item.get("category").unwrap().clone()).collect();
            let y_data: Vec<String> = data_bin.data.iter().map(|item| item.get("value1").unwrap().clone()).collect();
            chart_data.insert("x".to_string(), x_data);
            chart_data.insert("y".to_string(), y_data);
        }
        // Add more chart types and their corresponding data preparation logic
        _ => (),
    }

    chart_data
}

// Function to plot a simple Bokeh figure
pub fn plot_figure(data_bin: &DataBin, chart_type: &str, chart_config: &ChartConfig) {
    // Get the Bokeh module wrapped in a Mutex
    let bokeh = BOKEH.lock().unwrap();
    // Acquire the GIL and get the Python interpreter
    let gil = Python::acquire_gil();
    let py = gil.python();
    let plot = bokeh.getattr(py, "plotting").unwrap();
    let figure = plot.getattr(py, "figure").unwrap();
    let show = plot.getattr(py, "show").unwrap();
    let chart_data = prepare_data_for_chart(data_bin, chart_type);
    let fig = figure.call(py, (), None).unwrap();

    match chart_type {
        "line" => {
            let x = chart_data.get("x").unwrap();
            let y = chart_data.get("y").unwrap();
            fig.call_method(py, "line", (x, y), None).unwrap();
        }
        "bar" => {
            let x = chart_data.get("x").unwrap();
            let y = chart_data.get("y").unwrap();
            fig.call_method(py, "vbar", (x, y), None).unwrap();
        }
        // Add more chart types and their corresponding plotting logic
        _ => (),
    }

    // Apply chart configuration
    fig.call_method(py, "plot_width", (chart_config.plot_width,), None).unwrap();
    fig.call_method(py, "plot_height", (chart_config.plot_height,), None).unwrap();
    // Apply other chart configuration options

    show.call(py, (fig,), None).unwrap();
}

pub fn find_group(items: &[HashMap<String, String>], key: &str, reverse: bool) -> Vec<String> {
    let mut groups: Vec<String> = items
        .iter()
        .filter_map(|item| item.get(key).cloned())
        .collect();
    groups.sort();
    if reverse {
        groups.reverse();
    }
    let unique_groups: HashSet<String> = HashSet::from_iter(groups);
    let mut groups: Vec<String> = unique_groups.into_iter().collect();
    groups.sort();
    if reverse {
        groups.reverse();
    }
    groups
}

pub fn group_by(items: &[HashMap<String, String>], group_key: &str, reverse: bool) -> Vec<Vec<HashMap<String, String>>> {
    let groups = find_group(items, group_key, reverse);
    let mut results = Vec::new();
    for group in groups {
        let mut respective_data = Vec::new();
        for item in items {
            if let Some(value) = item.get(group_key) {
                if group == *value {
                    respective_data.push(item.clone());
                }
            }
        }
        results.push(respective_data);
    }
    results
}

pub fn ticker(tick: f32) -> String {
    format!("{:.0} + {:.2}", tick.trunc(), tick.fract())
}

pub fn group_commons(
    items: &[HashMap<String, String>],
    group: &str,
    group_first: &str,
    group_second: &str,
) -> Vec<Vec<HashMap<String, String>>> {
    let mut results = Vec::new();
    let groups = find_group(items, group, false);
    for g in groups {
        let mut respective_data = Vec::new();
        for item in items {
            if let Some(value) = item.get(group) {
                if g == *value {
                    if let Some(first_value) = item.get(group_first) {
                        if let Some(second_value) = item.get(group_second) {
                            if first_value == second_value {
                                respective_data.push(item.clone());
                            }
                        }
                    }
                }
            }
        }
        results.push(respective_data);
    }
    results
}

pub fn array_count(array: &[Vec<String>]) -> Vec<usize> {
    array.iter().map(|value| value.len()).collect()
}

pub fn array_sum(array: &[Vec<String>]) -> Vec<usize> {
    array.iter().map(|value| value.len()).collect()
}

pub fn array_average(array: &[Vec<String>]) -> Vec<usize> {
    array.iter().map(|value| value.len()).collect()
}

pub fn linear_scale_mixin(array: &[usize], multiply: usize, use_original: bool, scale: usize) -> Vec<usize> {
    let mut results = Vec::new();
    let mut m_l = Vec::new();
    let mut val = 0;
    for value in array {
        val += scale;
        if use_original {
            m_l.push(*value);
        } else {
            m_l.push(val);
        }
    }
    for _ in 0..multiply {
        results.extend(m_l.clone());
    }
    results
}
