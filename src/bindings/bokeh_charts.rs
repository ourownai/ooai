use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Item {
    pub block_size: String,
    pub graph_type: String,
    // Add other necessary fields for chart data
}

#[derive(Debug, Clone)]
pub struct ChartPlot {
    // Add necessary fields and types for chart plot
    pub plot_type: String,
    pub data: HashMap<String, Vec<f64>>,
    // Add other fields as needed
}

impl Default for ChartPlot {
    fn default() -> Self {
        ChartPlot {
            plot_type: "".to_string(),
            data: HashMap::new(),
            // Initialize other fields with default values
        }
    }
}

impl Item {
    fn prepare_line_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "line");
        ChartPlot {
            plot_type: "line".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_vbar_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "bar");
        ChartPlot {
            plot_type: "vbar".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_area_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "area");
        ChartPlot {
            plot_type: "area".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_hbar_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "hbar");
        ChartPlot {
            plot_type: "hbar".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_pie_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "pie");
        ChartPlot {
            plot_type: "pie".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_donut_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "donut");
        ChartPlot {
            plot_type: "donut".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_circle_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "circle");
        ChartPlot {
            plot_type: "circle".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_vbar_stack_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "vbar_stack");
        ChartPlot {
            plot_type: "vbar_stack".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_scatter_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "scatter");
        ChartPlot {
            plot_type: "scatter".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_heat_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "heat");
        ChartPlot {
            plot_type: "heat".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_hexbin_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "hexbin");
        ChartPlot {
            plot_type: "hexbin".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_interval_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "interval");
        ChartPlot {
            plot_type: "interval".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }

    fn prepare_candlestick_chart_data(&self, data_bin: &DataBin) -> ChartPlot {
        let chart_data = prepare_data_for_chart(data_bin, "candle");
        ChartPlot {
            plot_type: "candle".to_string(),
            data: chart_data,
            // Set other fields as needed
        }
    }
}

fn compute_bokeh_chart(item_ids: &[Item], data_bin: &DataBin) -> String {
    let mut grid_items: Vec<Vec<ChartPlot>> = Vec::new();

    if !item_ids.is_empty() {
        let number_of_two_col_items = item_ids
            .iter()
            .filter(|item| item.block_size == "2col")
            .count();
        let total_two_grid_requires = (number_of_two_col_items as f32 / 2.0).ceil() as usize;
        grid_items.extend(vec![vec![]; total_two_grid_requires]);

        let number_of_three_col_items = item_ids
            .iter()
            .filter(|item| item.block_size == "3col")
            .count();
        let total_three_grid_requires = (number_of_three_col_items as f32 / 3.0).ceil() as usize;
        grid_items.extend(vec![vec![]; total_three_grid_requires]);

        let mut two_col_count = 0;
        let mut two_col_grid_count = 0;
        let mut three_col_count = 0;
        let mut three_col_grid_count = 0;

        for item in item_ids {
            match item.block_size.as_str() {
                "1col" => grid_items.push(vec![append_chart_plot(item, data_bin)]),
                "2col" => {
                    grid_items[two_col_grid_count].push(append_chart_plot(item, data_bin));
                    two_col_count += 1;
                    if two_col_count == 2 {
                        two_col_grid_count += 1;
                        two_col_count = 0;
                    }
                }
                "3col" => {
                    grid_items[total_two_grid_requires + three_col_grid_count]
                        .push(append_chart_plot(item, data_bin));
                    three_col_count += 1;
                    if three_col_count == 3 {
                        three_col_grid_count += 1;
                        three_col_count = 0;
                    }
                }
                _ => (),
            }
        }

        // Call the `grid`, `components`, and other necessary functions here.
        let div_line = ""; // Replace with the appropriate function call
        let script_line = ""; // Replace with the appropriate function call
        let bokeh_chart = format!("{}{}", div_line, script_line);
        bokeh_chart
    } else {
        "".to_string()
    }
}

fn append_chart_plot(item: &Item, data_bin: &DataBin) -> ChartPlot {
    match item.graph_type.as_str() {
        "line" => item.prepare_line_chart_data(data_bin),
        "bar" => item.prepare_vbar_chart_data(data_bin),
        "area" => item.prepare_area_chart_data(data_bin),
        "hbar" => item.prepare_hbar_chart_data(data_bin),
        "pie" => item.prepare_pie_chart_data(data_bin),
        "donut" => item.prepare_donut_chart_data(data_bin),
        "circle" => item.prepare_circle_chart_data(data_bin),
        "vbar_stack" => item.prepare_vbar_stack_chart_data(data_bin),
        "scatter" => item.prepare_scatter_chart_data(data_bin),
        "heat" => item.prepare_heat_chart_data(data_bin),
        "hexbin" => item.prepare_hexbin_chart_data(data_bin),
        "interval" => item.prepare_interval_chart_data(data_bin),
        "candle" => item.prepare_candlestick_chart_data(data_bin),
        _ => ChartPlot::default(),
    }
}

// Add the missing `DataBin` struct and `prepare_data_for_chart` function here
pub struct DataBin {
    pub data: Vec<HashMap<String, String>>,
    pub fields: Vec<String>,
}

impl DataBin {
    pub fn new(data: Vec<HashMap<String, String>>, fields: Vec<String>) -> Self {
        DataBin { data, fields }
    }
}

pub fn prepare_data_for_chart(data_bin: &DataBin, chart_type: &str) -> HashMap<String, Vec<f64>> {
    let mut chart_data: HashMap<String, Vec<f64>> = HashMap::new();

    // Prepare data based on the chart type
    match chart_type {
        "line" | "bar" | "area" | "hbar" | "vbar_stack" | "scatter" | "heat" | "hexbin"
        | "interval" | "candle" => {
            let x_data: Vec<f64> = data_bin
                .data
                .iter()
                .map(|item| item.get("x").unwrap().parse().unwrap())
                .collect();
            let y_data: Vec<f64> = data_bin
                .data
                .iter()
                .map(|item| item.get("y").unwrap().parse().unwrap())
                .collect();
            chart_data.insert("x".to_string(), x_data);
            chart_data.insert("y".to_string(), y_data);
        }
        "pie" | "donut" => {
            let values: Vec<f64> = data_bin
                .data
                .iter()
                .map(|item| item.get("value").unwrap().parse().unwrap())
                .collect();
            chart_data.insert("values".to_string(), values);
        }
        "circle" => {
            let x_data: Vec<f64> = data_bin
                .data
                .iter()
                .map(|item| item.get("x").unwrap().parse().unwrap())
                .collect();
            let y_data: Vec<f64> = data_bin
                .data
                .iter()
                .map(|item| item.get("y").unwrap().parse().unwrap())
                .collect();
            let radius_data: Vec<f64> = data_bin
                .data
                .iter()
                .map(|item| item.get("radius").unwrap().parse().unwrap())
                .collect();
            chart_data.insert("x".to_string(), x_data);
            chart_data.insert("y".to_string(), y_data);
            chart_data.insert("radius".to_string(), radius_data);
        }
        _ => (),
    }

    chart_data
}