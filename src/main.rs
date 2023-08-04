//extern crate kiss3d;
//extern crate nalgebra as na;

use kiss3d::camera::{ArcBall, Camera, FixedView};
use kiss3d::event::{Event, WindowEvent};
use std::collections::HashSet;
use std::io::Cursor;
use wasm_bindgen::prelude::*;

use kiss3d::light::Light;
use kiss3d::nalgebra::{Point, Point2, Point3, UnitQuaternion, Vector2, Vector3};
use kiss3d::ncollide3d::procedural::{IndexBuffer, TriMesh};
use kiss3d::planar_camera::PlanarCamera;
use kiss3d::post_processing::PostProcessingEffect;
use kiss3d::renderer::Renderer;
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};
use log::info;
use stl_io::Vector;
use triangles::prelude::{IndexedTriangleList, Triangle3d, TriangleTopology};

struct AppState<'a> {
    c: SceneNode,
    camera: ArcBall,
    rot: UnitQuaternion<f32>,
    last_pos: Point2<f32>,
    triangles: IndexedTriangleList<Vector<f32>>,
    topology: TriangleTopology<'a, Vector<f32>>,
}

impl State for AppState<'_> {
    fn step(&mut self, window: &mut Window) {
        for event in window.events().iter() {
            match event.value {
                WindowEvent::Pos(_, _) => {}
                WindowEvent::Size(_, _) => {}
                WindowEvent::Close => {}
                WindowEvent::Refresh => {}
                WindowEvent::Focus(_) => {}
                WindowEvent::Iconify(_) => {}
                WindowEvent::FramebufferSize(_, _) => {}
                WindowEvent::MouseButton(button, action, modif) => {
                    info!("mouse press event on {:?} with {:?}", button, modif);
                    let window_size =
                        Vector2::new(window.size()[0] as f32, window.size()[1] as f32);
                    let sel_pos = self.camera.unproject(&self.last_pos, &window_size);
                    info!(
                        "conv {:?} to {:?} win siz {:?} ",
                        self.last_pos, sel_pos, window_size
                    );
                    let camera_pos = self.camera.eye();
                    info!("Button: {button:?}, action: {action:?}, {camera_pos}");
                }
                WindowEvent::CursorPos(x, y, modifiers) => {
                    self.last_pos = Point2::new(x as f32, y as f32);
                    //self.camera.un
                    //info!("x: {x}, y:{y}, m: {modifiers:?}")
                }
                WindowEvent::CursorEnter(_) => {}
                WindowEvent::Scroll(_, _, _) => {}
                WindowEvent::Key(_, _, _) => {}
                WindowEvent::Char(_) => {}
                WindowEvent::CharModifiers(_, _) => {}
                WindowEvent::Touch(_, _, _, _, _) => {}
            }
        }
    }

    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn Renderer>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        info!("cameras_and_effect_and_renderer");
        (Some(&mut self.camera), None, None, None)
    }
}
#[cfg(not(debug_assertions))]
const LOG_LEVEL: log::Level = log::Level::Info;
#[cfg(debug_assertions)]
const LOG_LEVEL: log::Level = log::Level::Trace;
pub fn main() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::new(LOG_LEVEL));

    let bytes = include_bytes!("Schublade - Front.stl");
    let mut cursor = Cursor::new(bytes);
    let stl_mesh = stl_io::read_stl(&mut cursor).expect("Cannot load stl file");
    let triangle_list: IndexedTriangleList<_> = stl_mesh.into();
    let topology = TriangleTopology::new(&triangle_list).expect("Errors in topology");
    let mesh: Vec<_> = topology
        .triangles_of_plane()
        .iter()
        .map(|(plane, triangles)| {
            let normal = plane.normal();
            //let mut point_map = HashMap::new();
            let used_points: HashSet<_> = triangles
                .iter()
                .flat_map(|tr| tr.points())
                .map(|p| p.idx())
                .collect();
            let mut reverse_map = vec![0; triangle_list.points().len()].into_boxed_slice();
            let mut coords: Vec<Point<f32, 3>> = vec![Point3::default(); used_points.len()];
            let points = triangle_list.points();
            for (target, source) in used_points.iter().enumerate() {
                reverse_map[*source] = target;
                let p = points[*source];
                coords[target] = Point3::from([p[0], p[1], p[2]]);
            }
            let faces: Vec<_> = triangles
                .iter()
                .map(|tr| Point3::from(tr.points().map(|p| reverse_map[p.idx()] as u32)))
                .collect();
            let mesh = TriMesh::new(coords, None, None, Some(IndexBuffer::Unified(faces)));
            mesh
        })
        .collect();

    let mut window = Window::new("Kiss3d: wasm example");
    window.set_background_color(1.0, 1.0, 1.0);
    let mut c = window.add_cube(0.1, 0.1, 0.1);

    c.set_color(0.8, 0.8, 2.0);

    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);
    let camera = ArcBall::new(Point3::new(0.0, 0.5, -1.0), Point3::new(0.0, 0.0, 0.0));
    let state = AppState {
        c,
        rot,
        camera,
        last_pos: Point2::new(0.0, 0.0),
        triangles: triangle_list,
    };

    window.render_loop(state);
    Ok(())
}

pub const WIN_W: u32 = 600;
pub const WIN_H: u32 = 420;
