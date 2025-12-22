// User interface logic - setup, drawing, formatting.

use crate::config::{ICON_NAME, PROGRAM_NAME, SETTINGSFILE, Units, load_config};
use crate::data::{
    GraphAttributes, GraphCache, MapCache, cvt_altitude, cvt_distance, cvt_elapsed_time, cvt_pace,
    cvt_temperature, get_run_start_date, get_sess_record_field, get_time_in_zone_field,
    get_timestamps, get_xy, is_american_thanksgiving, is_easter, semi_to_degrees, set_plot_range,
};
use crate::i18n::tr;
use directories::BaseDirs;
use fitparser::{FitDataField, FitDataRecord, profile::field_types::MesgNum};
use gtk4::cairo::Context;
use gtk4::ffi::GTK_STYLE_PROVIDER_PRIORITY_APPLICATION;
use gtk4::glib::clone;
use gtk4::prelude::*;
use gtk4::{
    Adjustment, Application, ApplicationWindow, Button, DrawingArea, DropDown, Frame, Image, Label,
    Orientation, Scale, ScrolledWindow, StringList, StringObject, TextBuffer, TextView, gdk,
};
use libshumate::prelude::*;
use libshumate::{Coordinate, Marker, MarkerLayer, PathLayer, SimpleMap};
use plotters::prelude::*;
use plotters::style::full_palette::BROWN;
use plotters::style::full_palette::CYAN;
use plotters_cairo::CairoBackend;
use std::path::Path;
use std::rc::Rc;

// #####################################################################
// ##################### OVERALL UI FUNCTIONS ##########################
// #####################################################################
// Widgets used for the graphical user interface.
pub struct UserInterface {
    pub settings_file: String,
    pub win: ApplicationWindow,
    pub outer_box: gtk4::Box,
    pub button_box: gtk4::Box,
    pub main_pane: gtk4::Paned,
    pub btn: Button,
    pub text_view: TextView,
    pub text_buffer: TextBuffer,
    pub frame_left: Frame,
    pub frame_right: Frame,
    pub left_frame_pane: gtk4::Paned,
    pub right_frame_pane: gtk4::Paned,
    pub scrolled_window: ScrolledWindow,
    pub map: libshumate::SimpleMap,
    pub path_layer: Option<PathLayer>,
    pub startstop_layer: Option<MarkerLayer>,
    pub marker_layer: Option<MarkerLayer>,
    pub da_window: ScrolledWindow,
    pub curr_pos_adj: Adjustment,
    pub curr_pos_scale: Scale,
    pub y_zoom_adj: Adjustment,
    pub x_zoom_adj: Adjustment,
    pub y_zoom_scale: Scale,
    pub curr_pos_label: Label,
    pub curr_time_label: Label,
    pub y_zoom_label: Label,
    pub controls_box: gtk4::Box,
    pub uom: StringList,
    pub units_widget: DropDown,
    pub about_label: String,
    pub about_btn: Button,
    pub da: DrawingArea,
}

// Instantiate the object holding the widgets (views).
pub fn instantiate_ui(app: &Application) -> UserInterface {
    let mut ui = UserInterface {
        settings_file: String::from(SETTINGSFILE),
        win: ApplicationWindow::builder()
            .application(app)
            .title(PROGRAM_NAME)
            .build(),
        // Main horizontal container to hold the two frames side-by-side,
        // outer box wraps main_pane.
        outer_box: gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .spacing(10)
            .build(),
        button_box: gtk4::Box::builder()
            .orientation(Orientation::Horizontal)
            .vexpand(false)
            .hexpand(false)
            .width_request(200)
            .height_request(20)
            .spacing(10)
            .build(),
        main_pane: gtk4::Paned::builder().build(),
        btn: Button::builder()
            .margin_top(5)
            .margin_bottom(5)
            .margin_start(5)
            .margin_end(5)
            .height_request(30)
            .width_request(50)
            .build(),
        text_view: TextView::builder()
            .monospace(true)
            .editable(false)
            .left_margin(25)
            .right_margin(25)
            .build(),
        text_buffer: TextBuffer::builder().build(),
        frame_left: Frame::builder().margin_bottom(5).build(),
        frame_right: Frame::builder().build(),
        left_frame_pane: gtk4::Paned::builder()
            .orientation(Orientation::Vertical)
            .margin_end(5)
            .shrink_start_child(true)
            .shrink_end_child(true)
            .resize_start_child(true)
            .resize_end_child(true)
            .build(),
        right_frame_pane: gtk4::Paned::builder()
            .orientation(Orientation::Horizontal)
            .margin_start(5)
            .shrink_start_child(true)
            .shrink_end_child(false)
            .resize_start_child(true)
            .resize_end_child(false)
            .build(),
        scrolled_window: ScrolledWindow::builder().margin_top(5).build(),
        map: SimpleMap::new(),
        path_layer: None,
        startstop_layer: None,
        marker_layer: None,
        da_window: ScrolledWindow::builder()
            .vexpand(true)
            .hexpand(true)
            .build(),
        curr_pos_adj: Adjustment::builder()
            .lower(0.0)
            .upper(1.0)
            .step_increment(0.001)
            .page_increment(0.001)
            .value(0.01)
            .build(),
        curr_pos_scale: Scale::builder()
            .orientation(Orientation::Horizontal)
            .draw_value(false)
            .vexpand(false)
            .width_request(120)
            .height_request(30)
            .build(),
        x_zoom_adj: Adjustment::builder()
            .lower(0.5)
            .upper(2.0)
            .step_increment(0.1)
            .page_increment(0.1)
            .value(1.0)
            .build(),
        y_zoom_adj: Adjustment::builder()
            .lower(0.5)
            .upper(4.0)
            .step_increment(0.1)
            .page_increment(0.1)
            .value(1.0)
            .build(),
        y_zoom_scale: Scale::builder()
            .orientation(Orientation::Horizontal)
            .draw_value(false)
            .vexpand(false)
            .width_request(120)
            .height_request(30)
            .build(),
        curr_pos_label: Label::new(Some("🏃‍➡️")),
        curr_time_label: Label::new(Some("")),
        y_zoom_label: Label::new(Some("🔍")),
        //        controls_box: gtk4::Box::new(Orientation::Vertical, 10),
        controls_box: gtk4::Box::builder()
            .orientation(Orientation::Vertical)
            .width_request(120)
            .spacing(10)
            .build(),
        uom: StringList::new(&["⚽ Metric", "🏈 US"]),

        units_widget: DropDown::builder()
            .margin_top(5)
            .margin_bottom(5)
            .margin_start(5)
            .margin_end(5)
            .height_request(30)
            .width_request(100)
            .build(),
        about_label: tr("about-button-label", None),
        about_btn: Button::builder()
            .margin_top(5)
            .margin_bottom(5)
            .margin_start(5)
            .margin_end(5)
            .height_request(30)
            .width_request(50)
            .build(),
        da: DrawingArea::builder().width_request(400).build(),
    };
    let provider = gtk4::CssProvider::new();
    let css_data =
        "textview { font: 14px monospace; font-weight: 500; color: black; background: white; }";
    provider.load_from_data(css_data);
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not get default display."),
        &provider,
        GTK_STYLE_PROVIDER_PRIORITY_APPLICATION.try_into().unwrap(),
    );
    ui.curr_pos_scale.set_adjustment(&ui.curr_pos_adj);
    ui.y_zoom_scale.set_adjustment(&ui.y_zoom_adj);
    ui.about_btn.set_label(&ui.about_label);
    ui.units_widget.set_model(Some(&ui.uom));
    ui.text_view.set_buffer(Some(&ui.text_buffer));
    ui.text_view.set_tooltip_text(Some("This section contains a report of lap, heart rate zones, and session summary information.\n\nThe text may be cut and pasted to other applications."));
    ui.scrolled_window.set_child(Some(&ui.text_view));
    ui.about_btn.set_tooltip_text(Some(
        "Show program credits, license, and copyright information.",
    ));

    // Button with icon and label.
    let button_content = gtk4::Box::new(Orientation::Horizontal, 6);
    button_content.set_halign(gtk4::Align::Center);
    // "document-open" is a standard Freedesktop icon name.
    let icon = Image::from_icon_name("document-open");
    let label = Label::new(Some("Open a FIT file..."));
    button_content.append(&icon);
    button_content.append(&label);
    ui.btn.set_child(Some(&button_content));
    ui.btn.set_tooltip_text(Some(
        "Open a Garmin Activity Fit file.\n\nPlease ensure you have copied the file from the watch to the file system first.",
    ));

    ui.units_widget.set_tooltip_text(Some(
        "Select your preferred unit system.\n\nThis will be the default next time you start the program but can be changed anytime.",
    ));
    ui.win.set_icon_name(Some(ICON_NAME));
    ui.win.set_child(Some(&ui.outer_box));
    ui.button_box.append(&ui.btn);
    ui.button_box.append(&ui.units_widget);
    ui.button_box.append(&ui.about_btn);
    ui.outer_box.append(&ui.button_box);
    ui.outer_box.append(&ui.main_pane);
    ui.controls_box.append(&ui.y_zoom_label);
    ui.controls_box.append(&ui.y_zoom_scale);
    ui.controls_box.append(&ui.curr_pos_label);
    ui.controls_box.append(&ui.curr_pos_scale);
    ui.controls_box.append(&ui.curr_time_label);
    ui.path_layer = Some(add_path_layer_to_map(&ui.map).unwrap());
    ui.startstop_layer = Some(add_marker_layer_to_map(&ui.map).unwrap());
    ui.marker_layer = Some(add_marker_layer_to_map(&ui.map).unwrap());

    ui.curr_pos_scale.set_tooltip_text(Some("Move from the beginning to the end of your run with this control.\n\nHairlines will appear on the graphs and a marker will appear on the map indicating your position. Reset to the beginning to remove.\n\nUse your keyboard right and left arrows for high precision control."));
    ui.curr_pos_label.set_tooltip_text(Some("Move from the beginning to the end of your run with this control.\n\nHairlines will appear on the graphs and a marker will appear on the map indicating your position. Reset to the beginning to remove.\n\nUse your keyboard right and left arrows for high precision control."));
    ui.y_zoom_scale
        .set_tooltip_text(Some("Zoom the graphs' y-axes with this control."));
    ui.y_zoom_label
        .set_tooltip_text(Some("Zoom the graphs' y-axes with this control."));
    ui.frame_left.set_tooltip_text(Some("This section displays a run path based on GPS data collected by your watch.\n\nLook carefully at the runner displayed during certain holidays to see a program Easter Egg."));
    ui.frame_right.set_tooltip_text(Some("This section contains graphs of values collected by your watch during your activity.\n\nDisplayed values are dependent on the sensors your watch supports (heart rate, altimeter, etc.)."));
    // query paths of user-invisible standard directories.
    let base_dirs = BaseDirs::new();
    if base_dirs.is_some() {
        ui.settings_file = base_dirs
            .unwrap()
            .config_dir()
            .join(SETTINGSFILE)
            .to_string_lossy()
            .to_string();
    }
    set_up_user_defaults(&ui);
    return ui;
}

// After reading the fit file, display the additional views of the UI.
pub fn construct_views_from_data(
    ui: &UserInterface,
    data: &Vec<FitDataRecord>,
    mc: &Rc<MapCache>,
    gc: &Rc<GraphCache>,
) {
    // 1. Instantiate embedded widgets based on parsed fit data.
    update_map_graph_and_summary_widgets(&ui, &data, &mc, &gc);

    // 2. Connect embedded widgets to their parents.
    ui.da_window.set_child(Some(&ui.da));
    ui.frame_right.set_child(Some(&ui.da_window));

    ui.frame_left.set_child(Some(&ui.map));
    // 3. Configure the widget layout.
    ui.left_frame_pane.set_start_child(Some(&ui.frame_left));
    ui.left_frame_pane.set_end_child(Some(&ui.scrolled_window));
    ui.right_frame_pane.set_start_child(Some(&ui.frame_right));
    ui.right_frame_pane.set_end_child(Some(&ui.controls_box));
    // Main box contains all of the above plus the graphs.
    ui.main_pane.set_start_child(Some(&ui.left_frame_pane));
    ui.main_pane.set_end_child(Some(&ui.right_frame_pane));

    // 4. Size the widgets.
    ui.scrolled_window.set_size_request(500, 300);
}

// Connect up the interactive widget handlers.
pub fn connect_interactive_widgets(
    ui: &Rc<UserInterface>,
    data: &Vec<FitDataRecord>,
    mc_rc: &Rc<MapCache>,
    gc_rc: &Rc<GraphCache>,
) {
    // clone the Rc pointer for each independent closure that needs the data.
    let mc_rc_for_units = Rc::clone(&mc_rc);
    // Hook-up the units_widget change handler.
    // update everything when the unit system changes.
    ui.units_widget.connect_selected_notify(clone!(
        #[strong]
        data,
        #[strong]
        ui,
        move |_| {
            // Create a new graph cache due to unit change.
            let graph_cache_units = instantiate_graph_cache(&data, &ui);
            // Wrap the GraphCache in an Rc for shared ownership.
            let gc_rc_for_units = Rc::new(graph_cache_units);
            update_map_graph_and_summary_widgets(&ui, &data, &mc_rc_for_units, &gc_rc_for_units);
            let curr_pos = ui.curr_pos_adj.clone();
            update_marker_layer(&data, &ui, &curr_pos, &mc_rc_for_units);
            // ui.map.queue_draw();
            ui.da.queue_draw();
        },
    ));

    // Hook-up the zoom scale change handler.
    // redraw the graphs when the zoom changes.
    ui.y_zoom_scale.adjustment().connect_value_changed(clone!(
        #[strong]
        data,
        #[strong]
        ui,
        move |_| {
            // Create a new graph cache due to zoom.
            let graph_cache_zoom = instantiate_graph_cache(&data, &ui);
            // Wrap the GraphCache in an Rc for shared ownership.
            let gc_rc_for_zoom = Rc::new(graph_cache_zoom);
            build_graphs(&data, &ui, &gc_rc_for_zoom);
            ui.da.queue_draw();
        },
    ));

    // Hook-up the current position change handler.
    // redraw the graphs and map when the current position changes.
    // clone the Rc pointer for each independent closure that needs the data.
    let mc_rc_for_marker = Rc::clone(&mc_rc);
    let gc_rc_for_scale = Rc::clone(&gc_rc);
    let curr_pos = ui.curr_pos_adj.clone();
    ui.curr_pos_scale.adjustment().connect_value_changed(clone!(
        #[strong]
        data,
        #[strong]
        ui,
        #[strong]
        curr_pos,
        move |_| {
            // Update timestamp
            update_timestamp(&ui, &curr_pos, &gc_rc_for_scale);
            // Update graphs.
            ui.da.queue_draw();
            // Update marker.
            update_marker_layer(&data, &ui, &curr_pos, &mc_rc_for_marker);
            // Update map.
            ui.map.queue_draw();
        },
    ));
}

// Return a unit enumeration from a units widget.
pub fn get_unit_system(units_widget: &DropDown) -> Units {
    if units_widget.model().is_some() {
        let model = units_widget.model().unwrap();
        if let Some(item_obj) = model.item(units_widget.selected()) {
            if let Ok(string_obj) = item_obj.downcast::<StringObject>() {
                let unit_string = String::from(string_obj.string());
                if unit_string == "⚽ Metric" {
                    return Units::Metric;
                }
                if unit_string == "🏈 US" {
                    return Units::US;
                }
            }
        }
    }
    return Units::None;
}

// Load the application settings from a configuration file.
pub fn set_up_user_defaults(ui: &UserInterface) {
    let config = load_config(&Path::new(&ui.settings_file));
    ui.win.set_default_size(config.width, config.height);
    ui.main_pane.set_position(config.main_split);
    ui.right_frame_pane.set_position(config.right_frame_split);
    ui.left_frame_pane.set_position(config.left_frame_split);
    ui.units_widget.set_selected(config.units_index);
}

// #####################################################################
// ##################### GRAPH FUNCTIONS ###############################
// #####################################################################
// Use plotters.rs to draw a graph on the drawing area.
fn draw_graphs(
    gc_rc: &Rc<GraphCache>,
    curr_adj: &Adjustment,
    cr: &Context,
    width: f64,
    height: f64,
) {
    // --- 🎨 Custom Drawing Logic Starts Here ---
    let gc = &**gc_rc;
    let root = plotters_cairo::CairoBackend::new(&cr, (width as u32, height as u32))
        .unwrap()
        .into_drawing_area();
    let _ = root.fill(&WHITE);
    let areas = root.split_evenly((2, 3));
    // Declare and initialize.
    for (a, idx) in areas.iter().zip(1..) {
        // After this point, we should be able to construct a chart context
        if idx == 1 {
            if gc.distance_pace.plotvals.len() == 0 {
                continue;
            };
            build_individual_graph(
                &gc.distance_pace.plotvals,
                gc.distance_pace.caption.as_str(),
                gc.distance_pace.xlabel.as_str(),
                gc.distance_pace.ylabel.as_str(),
                &gc.distance_pace.plot_range,
                &gc.distance_pace.y_formatter,
                &GREEN,
                curr_adj,
                a,
            );
        }
        if idx == 2 {
            if gc.distance_heart_rate.plotvals.len() == 0 {
                continue;
            };
            build_individual_graph(
                &gc.distance_heart_rate.plotvals,
                gc.distance_heart_rate.caption.as_str(),
                gc.distance_heart_rate.xlabel.as_str(),
                gc.distance_heart_rate.ylabel.as_str(),
                &gc.distance_heart_rate.plot_range,
                &gc.distance_heart_rate.y_formatter,
                &BLUE,
                curr_adj,
                a,
            )
        }
        if idx == 3 {
            if gc.distance_cadence.plotvals.len() == 0 {
                continue;
            };
            build_individual_graph(
                &gc.distance_cadence.plotvals,
                gc.distance_cadence.caption.as_str(),
                gc.distance_cadence.xlabel.as_str(),
                gc.distance_cadence.ylabel.as_str(),
                &gc.distance_cadence.plot_range,
                &gc.distance_cadence.y_formatter,
                &CYAN,
                curr_adj,
                a,
            )
        }
        if idx == 4 {
            if gc.distance_elevation.plotvals.len() == 0 {
                continue;
            };
            build_individual_graph(
                &gc.distance_elevation.plotvals,
                gc.distance_elevation.caption.as_str(),
                gc.distance_elevation.xlabel.as_str(),
                gc.distance_elevation.ylabel.as_str(),
                &gc.distance_elevation.plot_range,
                &gc.distance_elevation.y_formatter,
                &RED,
                curr_adj,
                a,
            )
        }
        if idx == 5 {
            if gc.distance_temperature.plotvals.len() == 0 {
                continue;
            };
            build_individual_graph(
                &gc.distance_temperature.plotvals,
                gc.distance_temperature.caption.as_str(),
                gc.distance_temperature.xlabel.as_str(),
                gc.distance_temperature.ylabel.as_str(),
                &gc.distance_temperature.plot_range,
                &gc.distance_temperature.y_formatter,
                &BROWN,
                curr_adj,
                a,
            )
        }
        if idx == 6 {
            break;
        }
    }

    let _ = root.present();
    // --- Custom Drawing Logic Ends Here ---
}

// Use plotters to actually draw a graph.
fn build_individual_graph(
    plotvals: &Vec<(f32, f32)>,
    caption: &str,
    xlabel: &str,
    ylabel: &str,
    plot_range: &(std::ops::Range<f32>, std::ops::Range<f32>),
    y_formatter: &Box<dyn Fn(&f32) -> String>,
    color: &RGBColor,
    curr_adj: &Adjustment,
    a: &plotters::drawing::DrawingArea<CairoBackend<'_>, plotters::coord::Shift>,
) {
    let mut chart = ChartBuilder::on(&a)
        // Set the caption of the chart
        .caption(caption, ("sans-serif", 16).into_font())
        // Set the size of the label region
        .x_label_area_size(40)
        .y_label_area_size(60)
        .margin(10)
        // Finally attach a coordinate on the drawing area and make a chart context
        .build_cartesian_2d(plot_range.clone().0, plot_range.clone().1)
        .unwrap();
    let _ = chart
        .configure_mesh()
        // We can customize the maximum number of labels allowed for each axis
        .x_labels(5)
        .y_labels(5)
        .x_desc(xlabel)
        .y_desc(ylabel)
        .y_label_formatter(&y_formatter)
        .draw();
    // // And we can draw something in the drawing area
    // We need to clone plotvals each time we make a call to LineSeries and PointSeries
    let _ = chart.draw_series(LineSeries::new(plotvals.clone(), color));
    // Calculate the hairline.
    let idx = (curr_adj.value() * (plotvals.len() as f64 - 1.0)).trunc() as usize;
    if idx > 0 && idx < plotvals.len() - 1 {
        let hair_x = plotvals[idx].0;
        let hair_y = plotvals[idx].1;
        let mylabel = format!(
            "{:<1}: {:<5.2}{:<1}: {:>1}",
            xlabel,
            hair_x,
            ylabel,
            &y_formatter(&hair_y)
        )
        .to_string();
        let hair_y_min = plot_range.clone().0.start;
        let hair_y_max = plot_range.clone().1.end;
        let mut hairlinevals: Vec<(f32, f32)> = Vec::new();
        hairlinevals.push((hair_x, hair_y_min));
        hairlinevals.push((hair_x, hair_y_max));
        let _ = chart
            .draw_series(DashedLineSeries::new(
                hairlinevals,
                1,
                4,
                ShapeStyle {
                    color: BLACK.mix(1.0),
                    filled: false,
                    stroke_width: 1,
                },
            ))
            .unwrap()
            .label(mylabel);

        chart
            .configure_series_labels()
            .position(SeriesLabelPosition::UpperLeft)
            .margin(5)
            .legend_area_size(0)
            .label_font(("Calibri", 10))
            .draw()
            .unwrap();
    }
}
// Build the graphs.  Prepare the graphical data for the drawing area and
// set-up the draw function callback.
fn build_graphs(_data: &Vec<FitDataRecord>, ui: &UserInterface, gc_rc: &Rc<GraphCache>) {
    // Need to clone to use inside the closure.
    let curr_pos = ui.curr_pos_adj.clone();
    ui.da.set_draw_func(clone!(
        #[strong]
        gc_rc,
        move |_drawing_area, cr, width, height| {
            draw_graphs(&gc_rc, &curr_pos, cr, width as f64, height as f64);
        }
    ));
}

// Update the views when supplied with data.
fn update_map_graph_and_summary_widgets(
    ui: &UserInterface,
    data: &Vec<FitDataRecord>,
    mc_rc: &Rc<MapCache>,
    gc_rc: &Rc<GraphCache>,
) {
    build_map(&data, &ui, &mc_rc);
    build_graphs(&data, &ui, &gc_rc);
    build_summary(&data, &ui);
    return;
}

// #####################################################################
// ##################### MAP FUNCTIONS #################################
// #####################################################################
// Add a marker layer to the map.
fn add_marker_layer_to_map(map: &SimpleMap) -> Option<MarkerLayer> {
    if map.viewport().is_some() {
        let viewport = map.viewport().unwrap();
        let marker_layer = libshumate::MarkerLayer::new(&viewport);
        map.add_overlay_layer(&marker_layer);
        return Some(marker_layer.clone());
    }
    return None;
}

// Adds a PathLayer with a path of given coordinates to the map.
fn add_path_layer_to_map(map: &SimpleMap) -> Option<PathLayer> {
    if map.viewport().is_some() {
        let viewport = map.viewport().unwrap();
        let path_layer = PathLayer::new(&viewport);
        let result = gdk::RGBA::parse("blue");
        match result {
            Ok(_) => {
                let blue = gdk::RGBA::parse("blue").unwrap();
                path_layer.set_stroke_color(Some(&blue));
            }
            Err(_) => {}
        }
        path_layer.set_stroke_width(2.0); // Thickness in pixels
        // Add the layer to the map
        map.add_overlay_layer(&path_layer);
        return Some(path_layer.clone());
    }
    return None;
}
// Return a (date dependent) unicode symbol.
fn get_symbol(data: &Vec<FitDataRecord>) -> &str {
    let mut symbol = concat!(r#"<span size="200%">"#, "🏃", "</span>");
    let (year, month, day) = get_run_start_date(data);
    if month == 1 && day == 1 {
        symbol = concat!(r#"<span size="200%">"#, "🍾", "</span>");
    }
    if month == 3 && day == 17 {
        symbol = concat!(r#"<span size="200%">"#, "🍀", "</span>");
    }
    if month == 7 && day == 4 {
        symbol = concat!(r#"<span size="200%">"#, "🎆", "</span>");
    }
    if month == 10 && day == 31 {
        symbol = concat!(r#"<span size="200%">"#, "🎃", "</span>");
    }
    if month == 12 && day == 24 {
        symbol = concat!(r#"<span size="200%">"#, "🎅", "</span>");
    }
    if month == 12 && day == 25 {
        symbol = concat!(r#"<span size="200%">"#, "🎁", "</span>");
    }
    if month == 12 && day == 31 {
        symbol = concat!(r#"<span size="200%">"#, "🍾", "</span>");
    }
    if is_american_thanksgiving(year, month, day) {
        symbol = concat!(r#"<span size="200%">"#, "🦃", "</span>");
    }
    if is_easter(year, month, day) {
        symbol = concat!(r#"<span size="200%">"#, "🐰", "</span>");
    }
    let _ = "📍";
    return symbol;
}
// Update the displayed timestamp based on the slider.
fn update_timestamp(ui: &UserInterface, curr_pos: &Adjustment, gc_rc_for_scale: &GraphCache) {
    let idx =
        (curr_pos.value() * (gc_rc_for_scale.time_stamps.len() as f64 - 1.0)).trunc() as usize;
    if idx > 0 && idx < gc_rc_for_scale.time_stamps.len() {
        let timestamp = &gc_rc_for_scale.time_stamps[idx].to_string();
        ui.curr_time_label.set_text(&timestamp);
    }
}
// Move the marker based on the current position.
fn update_marker_layer(
    data: &Vec<FitDataRecord>,
    ui: &UserInterface,
    curr_pos: &Adjustment,
    mc: &MapCache,
) {
    ui.marker_layer.as_ref().unwrap().remove_all();
    let run_path = &mc.run_path;
    let idx = (curr_pos.value() * (run_path.len() as f64 - 1.0)).trunc() as usize;
    let curr_lat = run_path[idx].0;
    let curr_lon = run_path[idx].1;
    let lat_deg = semi_to_degrees(curr_lat);
    let lon_deg = semi_to_degrees(curr_lon);
    let marker_text = Some(get_symbol(&data));
    let marker_content = gtk4::Label::new(marker_text);
    marker_content.set_halign(gtk4::Align::Center);
    marker_content.set_valign(gtk4::Align::Baseline);
    // Style the symbol with mark-up language.
    marker_content.set_markup(get_symbol(&data));
    let widget = &marker_content;
    let marker = Marker::builder()
        //            .label()
        .latitude(lat_deg)
        .longitude(lon_deg)
        .child(&widget.clone())
        // Set the visual content widget
        .build();
    ui.marker_layer.as_ref().unwrap().add_marker(&marker);
}

// Build the map.
fn build_map(data: &Vec<FitDataRecord>, ui: &UserInterface, mc_rc: &Rc<MapCache>) {
    if libshumate::MapSourceRegistry::with_defaults()
        .by_id("osm-mapnik")
        .is_some()
    {
        let mc = &**mc_rc;
        let source = libshumate::MapSourceRegistry::with_defaults()
            .by_id("osm-mapnik")
            .unwrap();
        ui.map.set_map_source(Some(&source));
        // Get values from fit file.
        let run_path = &mc.run_path;
        ui.path_layer.as_ref().unwrap().remove_all();
        for (lat, lon) in run_path.clone() {
            let coord = Coordinate::new_full(semi_to_degrees(lat), semi_to_degrees(lon));
            ui.path_layer.as_ref().unwrap().add_node(&coord);
        }
        ui.map.add_overlay_layer(ui.path_layer.as_ref().unwrap());
        // add pins for the starting and stopping points of the run
        ui.startstop_layer.as_ref().unwrap().remove_all();
        let len = run_path.len();
        if len > 0 {
            let start_lat_deg = semi_to_degrees(run_path[0..1][0].0);
            let start_lon_deg = semi_to_degrees(run_path[0..1][0].1);
            let stop_lat_deg = semi_to_degrees(run_path[len - 1..len][0].0);
            let stop_lon_deg = semi_to_degrees(run_path[len - 1..len][0].1);
            let start_content = gtk4::Label::new(Some("🟢"));
            let stop_content = gtk4::Label::new(Some("🔴"));
            start_content.set_halign(gtk4::Align::Center);
            start_content.set_valign(gtk4::Align::Baseline);
            stop_content.set_halign(gtk4::Align::Center);
            stop_content.set_valign(gtk4::Align::Baseline);
            let start_widget = &start_content;
            let stop_widget = &stop_content;
            let start_marker = Marker::builder()
                .latitude(start_lat_deg)
                .longitude(start_lon_deg)
                .child(&start_widget.clone())
                // Set the visual content widget
                .build();
            let stop_marker = Marker::builder()
                .latitude(stop_lat_deg)
                .longitude(stop_lon_deg)
                .child(&stop_widget.clone())
                // Set the visual content widget
                .build();
            ui.startstop_layer
                .as_ref()
                .unwrap()
                .add_marker(&start_marker);
            ui.startstop_layer
                .as_ref()
                .unwrap()
                .add_marker(&stop_marker);
        }
        ui.map
            .add_overlay_layer(ui.startstop_layer.as_ref().unwrap());
        // Add a layer for indication of current position (aka the runner).
        ui.marker_layer.as_ref().unwrap().remove_all();
        ui.map.add_overlay_layer(ui.marker_layer.as_ref().unwrap());
        // You may want to set an initial center and zoom level.
        if ui.map.viewport().is_some() {
            let viewport = ui.map.viewport().unwrap();
            let nec_lat = get_sess_record_field(&data, "nec_lat");
            let nec_long = get_sess_record_field(&data, "nec_long");
            let swc_lat = get_sess_record_field(&data, "swc_lat");
            let swc_long = get_sess_record_field(&data, "swc_long");
            if !nec_lat.is_nan() & !nec_long.is_nan() & !swc_lat.is_nan() & !swc_long.is_nan() {
                let center_lat =
                    (semi_to_degrees(nec_lat as f32) + semi_to_degrees(swc_lat as f32)) / 2.0;
                let center_long =
                    (semi_to_degrees(nec_long as f32) + semi_to_degrees(swc_long as f32)) / 2.0;
                viewport.set_location(center_lat, center_long);
            } else {
                viewport.set_location(29.7601, -95.3701); // e.g. Houston, USA
            }
            viewport.set_zoom_level(14.0);
        }
    }
}

// #####################################################################
// ##################### SUMMARY FUNCTIONS #############################
// #####################################################################
// Convert a value to user-defined units and return a formatted string when supplied a field and units.
fn format_string_for_field(fld: &FitDataField, user_unit: &Units) -> Option<String> {
    match fld.name() {
        "start_position_lat" | "start_position_long" | "end_position_lat" | "end_position_long" => {
            let result: Result<i64, _> = fld.value().try_into();
            match result {
                Ok(semi) => {
                    let degrees = semi_to_degrees(semi as f32);
                    return Some(format!("{:<23}: {degrees:<6.3}°\n", fld.name(),));
                }
                Err(_) => return None,
            }
        }

        "total_strides"
        | "total_calories"
        | "avg_heart_rate"
        | "max_heart_rate"
        | "avg_running_cadence"
        | "max_running_cadence"
        | "total_training_effect"
        | "first_lap_index"
        | "num_laps"
        | "avg_fractional_cadence"
        | "max_fractional_cadence"
        | "total_anaerobic_training_effect"
        | "sport"
        | "sub_sport"
        | "timestamp"
        | "start_time" => {
            return Some(format!(
                "{:<23}: {:<#} {:<}\n",
                fld.name(),
                fld.value(),
                fld.units()
            ));
        }
        "total_ascent" | "total_descent" => {
            let result: Result<f64, _> = fld.value().clone().try_into();
            match result {
                Ok(val) => {
                    let val_cvt = cvt_altitude(val as f32, &user_unit);
                    match user_unit {
                        Units::US => {
                            return Some(format!(
                                "{:<23}: {:<.2} {:<}\n",
                                fld.name(),
                                val_cvt,
                                "feet"
                            ));
                        }
                        Units::Metric => {
                            return Some(format!(
                                "{:<23}: {:<.2} {:<}\n",
                                fld.name(),
                                val_cvt,
                                "meters"
                            ));
                        }
                        Units::None => {
                            return Some(format!("{:<23}: {:<.2} {:<}\n", fld.name(), val_cvt, ""));
                        }
                    }
                }
                Err(_) => return None,
            }
        }
        "total_distance" => {
            let result: Result<f64, _> = fld.value().clone().try_into();
            match result {
                Ok(val) => {
                    let val_cvt = cvt_distance(val as f32, &user_unit);
                    match user_unit {
                        Units::US => {
                            return Some(format!(
                                "{:<23}: {:<.2} {:<}\n",
                                fld.name(),
                                val_cvt,
                                "miles"
                            ));
                        }
                        Units::Metric => {
                            return Some(format!(
                                "{:<23}: {:<.2} {:<}\n",
                                fld.name(),
                                val_cvt,
                                "kilometers"
                            ));
                        }
                        Units::None => {
                            return Some(format!("{:<23}: {:<.2} {:<}\n", fld.name(), val_cvt, ""));
                        }
                    }
                }
                Err(_) => return None,
            }
        }
        "total_elapsed_time" | "total_timer_time" => {
            let result: Result<f64, _> = fld.value().clone().try_into();
            match result {
                Ok(val) => {
                    let val_cvt = cvt_elapsed_time(val as f32);
                    return Some(format!(
                        "{:<23}: {:01}h:{:02}m:{:02}s\n",
                        fld.name(),
                        val_cvt.0,
                        val_cvt.1,
                        val_cvt.2
                    ));
                }
                Err(_) => return None,
            }
        }
        "min_temperature" | "max_temperature" | "avg_temperature" => {
            let result: Result<i64, _> = fld.value().clone().try_into();
            match result {
                Ok(val) => {
                    let val_cvt = cvt_temperature(val as f32, &user_unit);
                    match user_unit {
                        Units::US => {
                            return Some(format!(
                                "{:<23}: {:<.2} {:<}\n",
                                fld.name(),
                                val_cvt,
                                "°F"
                            ));
                        }
                        Units::Metric => {
                            return Some(format!(
                                "{:<23}: {:<.2} {:<}\n",
                                fld.name(),
                                val_cvt,
                                "°C"
                            ));
                        }
                        Units::None => {
                            return Some(format!("{:<23}: {:<.2} {:<}\n", fld.name(), val_cvt, ""));
                        }
                    }
                }
                Err(_) => return None,
            }
        }
        "enhanced_avg_speed" | "enhanced_max_speed" => {
            let result: Result<f64, _> = fld.value().clone().try_into();
            match result {
                Ok(val) => {
                    let val_cvt = cvt_pace(val as f32, &user_unit);
                    match user_unit {
                        Units::US => {
                            return Some(format!(
                                "{:<23}: {:<.2} {:<}\n",
                                fld.name(),
                                val_cvt,
                                "min/mile"
                            ));
                        }
                        Units::Metric => {
                            return Some(format!(
                                "{:<23}: {:<.2} {:<}\n",
                                fld.name(),
                                val_cvt,
                                "min/km"
                            ));
                        }
                        Units::None => {
                            return Some(format!("{:<23}: {:<.2} {:<}\n", fld.name(), val_cvt, ""));
                        }
                    }
                }
                Err(_) => return None,
            }
        }
        _ => return None, // matches other patterns
    }
}

// Build a summary.
fn build_summary(data: &Vec<FitDataRecord>, ui: &UserInterface) {
    // Get the enumerated value for the unit system the user selected.
    let user_unit = get_unit_system(&ui.units_widget);
    ui.text_buffer.set_text("File loaded.");
    // Clear out anything in the buffer.
    let mut start = ui.text_buffer.start_iter();
    let mut end = ui.text_buffer.end_iter();
    ui.text_buffer.delete(&mut start, &mut end);
    let mut lap_index: u8 = 0;
    let mut lap_str: String;
    for item in data {
        match item.kind() {
            MesgNum::Session | MesgNum::Lap => {
                // print all the data records in FIT file
                if item.kind() == MesgNum::Session {
                    ui.text_buffer.insert(&mut end, "\n");
                    ui.text_buffer.insert(
                        &mut end,
                        "============================ Session ==================================\n",
                    );
                    ui.text_buffer.insert(&mut end, "\n");
                }
                if item.kind() == MesgNum::Lap {
                    lap_index = lap_index + 1;
                    lap_str = format!(
                        "------------------------------ Lap {}-----------------------------------\n",
                        lap_index
                    );
                    ui.text_buffer.insert(&mut end, "\n");
                    ui.text_buffer.insert(&mut end, &lap_str);
                    ui.text_buffer.insert(&mut end, "\n");
                }
                // Retrieve the FitDataField struct.
                for fld in item.fields().iter() {
                    let value_str = format_string_for_field(fld, &user_unit);
                    if value_str.is_some() {
                        ui.text_buffer.insert(&mut end, &value_str.unwrap());
                    }
                }
            }
            _ => print!("{}", ""), // matches other patterns
        }
    }
    if let (Some(zone_times), Some(zone_limits)) = get_time_in_zone_field(data) {
        // There are 7 zones but only 6 upper limits.
        ui.text_buffer.insert(&mut end, "\n");
        ui.text_buffer.insert(
            &mut end,
            "=================== Time in Heart Rate Zones for Session  ========\n",
        );
        ui.text_buffer.insert(&mut end, "\n");
        for (z, val) in zone_times.iter().enumerate() {
            let val_cvt = cvt_elapsed_time(*val as f32);
            let ll: f64;
            let ul: f64;
            if z == 0 {
                ll = 0.0;
                ul = zone_limits[z];
            } else if z < zone_limits.len() && z > 0 {
                ll = zone_limits[z - 1];
                ul = zone_limits[z];
            } else {
                ll = zone_limits[z - 1];
                ul = 220.0;
            }
            let value_str = format!(
                "{:<5}{:<} ({:>3}-{:>3} bpm): {:01}h:{:02}m:{:02}s\n",
                "Zone", z, ll as i32, ul as i32, val_cvt.0, val_cvt.1, val_cvt.2
            );
            ui.text_buffer.insert(&mut end, &value_str);
        }
        ui.text_buffer.insert(&mut end, "\n");
    };
}

// #####################################################################
// ##################### CACHE FUNCTIONS ###############################
// #####################################################################
// Calculate a cache of the graph attributes (see GraphAtributes) a *SINGLE* time for display.
pub fn instantiate_graph_cache(d: &Vec<FitDataRecord>, ui: &UserInterface) -> GraphCache {
    let user_unit = get_unit_system(&ui.units_widget);

    let zoom_x: f32 = ui.x_zoom_adj.value() as f32;
    let zoom_y: f32 = ui.y_zoom_adj.value() as f32;
    let num_formatter = |x: &f32| format!("{:7.2}", x);
    let pace_formatter = |x: &f32| {
        let mins = x.trunc();
        let secs = x.fract() * 60.0;
        format!("{:02.0}:{:02.0}", mins, secs)
    };
    let mut xlabel: &str;
    let mut ylabel: &str;
    // distance_pace
    let xy = get_xy(&d, &ui.units_widget, "distance", "enhanced_speed");
    let range = set_plot_range(&xy, zoom_x, zoom_y);
    match user_unit {
        Units::US => {
            ylabel = "Pace (min/mile)";
            xlabel = "Distance (miles)";
        }
        Units::Metric => {
            ylabel = "Pace (min/km)";
            xlabel = "Distance (km)";
        }
        Units::None => {
            ylabel = "";
            xlabel = "";
        }
    }
    let distance_pace = GraphAttributes {
        plotvals: (xy),
        caption: (String::from("Pace")),
        xlabel: (String::from(xlabel)),
        ylabel: (String::from(ylabel)),
        plot_range: (range),
        y_formatter: (Box::new(pace_formatter)),
        // color: (&RED),
    };
    // distance_heart_rate
    let xy = get_xy(&d, &ui.units_widget, "distance", "heart_rate");
    let range = set_plot_range(&xy.clone(), zoom_x, zoom_y);
    match user_unit {
        Units::US => {
            ylabel = "Heart rate (bpm)";
            xlabel = "Distance (miles)";
        }
        Units::Metric => {
            ylabel = "Heart rate (bpm)";
            xlabel = "Distance (km)";
        }
        Units::None => {
            ylabel = "";
            xlabel = "";
        }
    }
    let distance_heart_rate = GraphAttributes {
        plotvals: (xy),
        caption: (String::from("Heart rate")),
        xlabel: (String::from(xlabel)),
        ylabel: (String::from(ylabel)),
        plot_range: (range),
        y_formatter: (Box::new(num_formatter)),
        // color: (&BLUE),
    };
    // distance-cadence
    let xy = get_xy(&d, &ui.units_widget, "distance", "cadence");
    let range = set_plot_range(&xy.clone(), zoom_x, zoom_y);
    match user_unit {
        Units::US => {
            ylabel = "Cadence";
            xlabel = "Distance (miles)";
        }
        Units::Metric => {
            ylabel = "Cadence";
            xlabel = "Distance (km)";
        }
        Units::None => {
            ylabel = "";
            xlabel = "";
        }
    }
    let distance_cadence = GraphAttributes {
        plotvals: (xy),
        caption: (String::from("Cadence")),
        xlabel: (String::from(xlabel)),
        ylabel: (String::from(ylabel)),
        plot_range: (range),
        y_formatter: (Box::new(num_formatter)),
        // color: (&CYAN),
    };
    //distance-elevation
    let xy = get_xy(&d, &ui.units_widget, "distance", "enhanced_altitude");
    let range = set_plot_range(&xy.clone(), zoom_x, zoom_y);
    match user_unit {
        Units::US => {
            ylabel = "Elevation (feet)";
            xlabel = "Distance (miles)";
        }
        Units::Metric => {
            ylabel = "Elevation (m)";
            xlabel = "Distance (km)";
        }
        Units::None => {
            ylabel = "";
            xlabel = "";
        }
    }
    let distance_elevation = GraphAttributes {
        plotvals: (xy),
        caption: (String::from("Elevation")),
        xlabel: (String::from(xlabel)),
        ylabel: (String::from(ylabel)),
        plot_range: (range),
        y_formatter: (Box::new(num_formatter)),
        // color: (&RED),
    };
    // distance-temperature
    let xy = get_xy(&d, &ui.units_widget, "distance", "temperature");
    let range = set_plot_range(&xy.clone(), zoom_x, zoom_y);
    match user_unit {
        Units::US => {
            ylabel = "Temperature (°F)";
            xlabel = "Distance (miles)";
        }
        Units::Metric => {
            ylabel = "Temperature (°C)";
            xlabel = "Distance (km)";
        }
        Units::None => {
            ylabel = "";
            xlabel = "";
        }
    }
    let distance_temperature = GraphAttributes {
        plotvals: (xy),
        caption: (String::from("Temperature")),
        xlabel: (String::from(xlabel)),
        ylabel: (String::from(ylabel)),
        plot_range: (range),
        y_formatter: (Box::new(num_formatter)),
        // color: (&BROWN),
    };

    let time_stamps = get_timestamps(&d);
    let gc: GraphCache = GraphCache {
        distance_pace: distance_pace,
        distance_heart_rate: distance_heart_rate,
        distance_cadence: distance_cadence,
        distance_elevation: distance_elevation,
        distance_temperature: distance_temperature,
        time_stamps: time_stamps,
    };
    return gc;
}

// Calculate a means to capture the data in run_path a *SINGLE* time.
pub fn instantiate_map_cache(d: &Vec<FitDataRecord>) -> MapCache {
    let units_widget = DropDown::builder().build(); // bogus value - no units required for position
    let run_path = get_xy(&d, &units_widget, "position_lat", "position_long");
    let mc: MapCache = MapCache { run_path: run_path };
    return mc;
}
