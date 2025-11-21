use gtk4::prelude::*;
use plotters::prelude::*;
use gtk4::{Application, ApplicationWindow, DrawingArea};
use gtk4::glib::clone;

fn main() {
    let app = Application::builder().build();
    app.connect_activate(build_gui);
    app.run();
}

fn max_vec(vector : Vec<f32>) -> f32  {
    // Find the largest non-NaN in vector, or NaN otherwise:
    let v = vector.iter().cloned().fold(0./0., f32::max);
    return v;
}

fn min_vec(vector : Vec<f32>) -> f32  {
    // Find the largest non-NaN in vector, or NaN otherwise:
    let v = vector.iter().cloned().fold(0./0., f32::min);
    return v;
}

fn get_plot_range(data : Vec<(f32, f32)>) -> (std::ops::Range<f32>, std::ops::Range<f32>) {
    // Split vector of tuples into two vecs
    let (x, y): (Vec<_>, Vec<_>) = data.into_iter().map(|(a, b)| (a, b)).unzip();    
    // Find the range of the chart
    let xrange : std::ops::Range<f32> = min_vec(x.clone())..max_vec(x.clone());
    let yrange : std::ops::Range<f32> = min_vec(y.clone())..max_vec(y.clone());
    return (xrange, yrange);
}

fn build_gui(app: &Application){
    let win = ApplicationWindow::builder().application(app).default_width(1024).default_height(768).title("Test").build();
    let drawing_area: DrawingArea = DrawingArea::builder().build();

    // Initialize shared plot data for testing...these will be from fit file.
    let mut plotvals: Vec<(f32, f32)> = Vec::new();
    plotvals.push((1.0, 3.0));
    plotvals.push((5.0, 6.0));
    plotvals.push((7.0, 9.0));
    // This is equivalent.
    //let mut plotvals = vec![(10.0, 0.0), (5.0, 5.0), (8.0, 7.0)];

    // Use a "closure" (anonymous function?) as the drawing area draw_func.
    // We pass a strong reference to the plot data (aka plotvals).
    drawing_area.set_draw_func(clone!(#[strong] plotvals, move |_drawing_area, cr, width, height| {
        // --- 🎨 Custom Drawing Logic Starts Here ---
 
        let root = plotters_cairo::CairoBackend::new(&cr, (width.try_into().unwrap(), height.try_into().unwrap())).unwrap().into_drawing_area();
        let _ = root.fill(&WHITE);

        let root = root.margin(10, 10, 10, 10);

        //  Find the plot range (minx..maxx, miny..maxy)
        let plot_range = get_plot_range(plotvals.clone());
        
        // After this point, we should be able to construct a chart context
        //
       
        let mut chart = ChartBuilder::on(&root)
            // Set the caption of the chart
            .caption("This is our first plot", ("sans-serif", 40).into_font())
            // Set the size of the label region
            .x_label_area_size(20)
            .y_label_area_size(40)
            // Finally attach a coordinate on the drawing area and make a chart context
            .build_cartesian_2d(plot_range.0, plot_range.1).unwrap();

        // Then we can draw a mesh
        let _ = chart
            .configure_mesh()
            // We can customize the maximum number of labels allowed for each axis
            .x_labels(15)
            .y_labels(5)
            // We can also change the format of the label text
            .y_label_formatter(&|x| format!("{:.3}", x))
            .draw();

        // And we can draw something in the drawing area
        // We need to clone plotvals each time we make a call to LineSeries and PointSeries
        let _ = chart.draw_series(LineSeries::new(
              plotvals.clone(),
            &RED,
        ));
        // Similarly, we can draw point series
        let _ = chart.draw_series(PointSeries::of_element(
              plotvals.clone(),
            5,
            &RED,
            &|c, s, st| {
                return EmptyElement::at(c)    // We want to construct a composed element on-the-fly
                + Circle::new((0,0),s,st.filled()) // At this point, the new pixel coordinate is established
                + Text::new(format!("{:?}", c), (10, 0), ("sans-serif", 10).into_font());
            },
        ));
        
        let _ = root.present();
        // --- Custom Drawing Logic Ends Here ---
    }));

    win.set_child(Some(&drawing_area));
    win.present();
}
