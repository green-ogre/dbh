use std::io::Read;

use winny::prelude::*;
use winny::{
    app::core::{App, Schedule},
    cereal::{Deserialize, Serialize},
    ecs::Resource,
};

pub trait LoaderApp {
    fn save_load_resource<R: Serialize + Deserialize + Resource + Default>(&mut self) -> &mut Self;
}

impl LoaderApp for App {
    fn save_load_resource<R: Serialize + Deserialize + Resource + Default>(&mut self) -> &mut Self {
        self.add_systems(Schedule::StartUp, load_resource::<R>)
            .add_systems(Schedule::Exit, save_resource::<R>)
    }
}

fn load_resource<R: Serialize + Deserialize + Resource + Default>(mut commands: Commands) {
    let camera =
        if let Ok(f) = std::fs::File::open(format!("res/saved/{}", std::any::type_name::<R>())) {
            info!("deserializing [{}]", std::any::type_name::<R>());
            let mut bytes = Vec::new();
            std::io::BufReader::new(f).read_to_end(&mut bytes).unwrap();
            let mut d = Deserializer::new(&mut bytes);
            if let Some(val) = R::deserialize(&mut d) {
                val
            } else {
                error!("failed to deserialize [{}]", std::any::type_name::<R>());
                R::default()
            }
        } else {
            R::default()
        };

    commands.insert_resource(camera);
}

fn save_resource<R: Serialize + Deserialize + Resource + Default>(resource: Res<R>) {
    let mut bytes = Vec::new();
    let mut s = Serializer::new(&mut bytes);
    resource.serialize(&mut s);
    match std::fs::write(format!("res/saved/{}", std::any::type_name::<R>()), &bytes) {
        Ok(_) => (),
        Err(e) => error!("failed to serialize [{}]: {e}", std::any::type_name::<R>()),
    }
}
