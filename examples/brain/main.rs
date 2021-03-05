
use three_d::*;
use neuroformats::{read_surf, read_curv};
use ndarray::{Array2};
//use colorous::{Color, VIRIDIS};

fn a22vec<T: Clone>(a2 : Array2<T>) -> Vec<T> {
    let mut ve : Vec<T> = Vec::with_capacity(a2.len());
    
    for v in a2.iter() {
        ve.push((*v).clone());
    }
    ve
}

fn f32tou32(a: Vec<i32>) -> Vec<u32> {
    let mut u : Vec<u32> = Vec::with_capacity(a.len());

    for v in a.iter() {
        u.push(*v as u32);
    }
    u
}

fn vec32minmax(a : Vec<f32>) -> Vec<f32> {
        let mut curv_data_sorted = a.to_vec();
        curv_data_sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let min: f32 = curv_data_sorted[0];
        let max: f32 = curv_data_sorted[curv_data_sorted.len() - 1];
        vec![min, max]
}

fn apply_colormap(data: Vec<f32>, cmap: colorous::Gradient) -> Vec<colorous::Color> {

    let nan_color = colorous::Color{ r: 255 as u8, g: 255 as u8, b: 255 as u8};
    let mut colors : Vec<colorous::Color> = Vec::with_capacity(data.len());
    for v in data.iter() {
        if v.is_nan() {
            colors.push(nan_color);
        } else {
            colors.push(cmap.eval_continuous((*v).into()));
        }
    } 
    colors
}

fn colors_as_u8_4(colors : Vec<colorous::Color>, alpha: u8) -> Vec<u8> {
    let mut col_255 : Vec<u8> = Vec::with_capacity(colors.len() * 4);
    for v in colors.iter() {
        let rgb : [u8; 3] = (*v).into_array();
        for c in rgb.iter() {
            col_255.push(*c);
        }
        col_255.push(alpha as u8);
    } 
    col_255
}

fn scale_to_01(data: Vec<f32>) -> Vec<f32> {
    let mut scaled : Vec<f32> = Vec::with_capacity(data.len());
    for v in data.iter() {
        scaled.push(*v / 5.0); // TODO: implement this.
    } 
    let mm = vec32minmax(data);
    let dmin = mm[0];
    let dmax = mm[1];
    println!("TOD: implement scale_to_01");
    scaled
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let screenshot_path = if args.len() > 1 { Some(args[1].clone()) } else {None};

    let window = Window::new("Cortical thickness", Some((1280, 720))).unwrap();
    let context = window.gl();

    let scene_center = vec3(0.0, 0.0, 0.0);
    let scene_radius = 300.0; // TODO: compute mesh max entend
    let mut camera = Camera::new_perspective(&context, scene_center + scene_radius * vec3(0.6, 0.3, 1.0).normalize(), scene_center, vec3(0.0, 1.0, 0.0),
                                             degrees(45.0), window.viewport().aspect(), 0.1, 1000.0);

    // Read brain meshes for both hemis.
    let lh_white = read_surf("resources/subjects_dir/subject1/surf/lh.white").unwrap();
    let rh_white = read_surf("resources/subjects_dir/subject1/surf/rh.white").unwrap();

    // Read brain morphometry data
    let lh_curv = read_curv("resources/subjects_dir/subject1/surf/lh.thickness").unwrap();
    let rh_curv = read_curv("resources/subjects_dir/subject1/surf/rh.thickness").unwrap();

    // generate colors from morph data
    let gradient = colorous::VIRIDIS;
    let lh_brain_colors = colors_as_u8_4(apply_colormap(scale_to_01(lh_curv.data), gradient), 255 as u8);
    let rh_brain_colors = colors_as_u8_4(apply_colormap(scale_to_01(rh_curv.data), gradient), 255 as u8);


    //let lh_brain_colors: Vec<u8> = vec![255; lh_white.mesh.vertices.len() * 4];
    //let rh_brain_colors: Vec<u8> = vec![255; rh_white.mesh.vertices.len() * 4];

    let lh_cpu_mesh = CPUMesh {
        positions : a22vec(lh_white.mesh.vertices), colors : Some(lh_brain_colors), indices : Some(f32tou32
        (a22vec(lh_white.mesh.faces))), ..Default::default()
    };
    let lh_mesh = Mesh::new(&context, &lh_cpu_mesh).unwrap();

    let rh_cpu_mesh = CPUMesh {
        positions : a22vec(rh_white.mesh.vertices), colors : Some(rh_brain_colors), indices : Some(f32tou32
        (a22vec(rh_white.mesh.faces))), ..Default::default()
    };
    let rh_mesh = Mesh::new(&context, &rh_cpu_mesh).unwrap();

    // main loop
    let mut rotating = false;
    window.render_loop(move |frame_input|
    {
        camera.set_aspect(frame_input.viewport.aspect());

        for event in frame_input.events.iter() {
            match event {
                Event::MouseClick { state, button, .. } => {
                    rotating = *button == MouseButton::Left && *state == State::Pressed;
                },
                Event::MouseMotion { delta, .. } => {
                    if rotating {
                        camera.rotate_around_up(delta.0 as f32, delta.1 as f32);
                    }
                },
                Event::MouseWheel { delta, .. } => {
                    camera.zoom(delta.1 as f32);
                },
                _ => {}
            }
        }

        Screen::write(&context, &ClearState::color_and_depth(0.8, 0.8, 0.8, 1.0, 1.0), || {
            let transformation = Mat4::from_angle_y(radians((frame_input.accumulated_time * 0.0005) as f32));
            lh_mesh.render_color(RenderStates::default(), frame_input.viewport, &transformation, &camera)?;
            rh_mesh.render_color(RenderStates::default(), frame_input.viewport, &transformation, &camera)?;
            Ok(())
        }).unwrap();

        #[cfg(target_arch = "x86_64")]
        if let Some(ref path) = screenshot_path {
            use three_d::io::*;
            let pixels = Screen::read_color(&context, frame_input.viewport).unwrap();
            Saver::save_pixels(path, &pixels, frame_input.viewport.width, frame_input.viewport.height).unwrap();
            std::process::exit(1);
        }
    }).unwrap();
}
