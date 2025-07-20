// Copyright © 2021-2025
// Author: Antonio Caggiano <info@antoniocaggiano.eu>
// SPDX-License-Identifier: MIT

use std::sync::Arc;

use crate::*;

/// Represents a scene in the rendering context, containing multiple models and cameras.
pub struct RenderScene {
    models: Pack<RenderModel>,
    dev: Arc<Dev>,
}

impl RenderScene {
    /// Creates a new empty `RenderScene`.
    pub fn new(dev: &Arc<Dev>) -> Self {
        let mut models = Pack::new();
        models.push(RenderModel::default(dev));
        Self {
            models,
            dev: dev.clone(),
        }
    }

    /// Adds a model to the scene.
    pub fn push_model(&mut self, model: RenderModel) -> Handle<RenderModel> {
        self.models.push(model)
    }

    /// Clears all models from the scene, but keeps the first model which is the default.
    pub fn clear(&mut self) {
        self.models
            .resize_with(1, || RenderModel::default(&self.dev));
    }

    /// Returns a reference to the models in the scene.
    pub fn get_models(&self) -> &Pack<RenderModel> {
        &self.models
    }

    /// Returns a mutable reference to the models in the scene.
    pub fn get_models_mut(&mut self) -> &mut Pack<RenderModel> {
        &mut self.models
    }

    /// Returns a model by its handle.
    pub fn get_model(&self, hmodel: Handle<RenderModel>) -> Option<&RenderModel> {
        self.models.get(hmodel)
    }

    /// Returns a mutable reference to a model by its handle.
    pub fn get_model_mut(&mut self, hmodel: Handle<RenderModel>) -> Option<&mut RenderModel> {
        self.models.get_mut(hmodel)
    }

    /// Returns the default model handle, which is the first one in the pack.
    pub fn get_default_model_handle(&self) -> Handle<RenderModel> {
        Handle::new(0)
    }

    /// Returns the default model, which is the first one in the pack.
    pub fn get_default_model(&self) -> &RenderModel {
        self.models.get(self.get_default_model_handle()).unwrap()
    }

    /// Returns a mutable reference to the default model.
    pub fn get_default_model_mut(&mut self) -> &mut RenderModel {
        self.models
            .get_mut(self.get_default_model_handle())
            .unwrap()
    }

    /// Returns the handle of the node with the default camera.
    pub fn get_default_camera_node_handle(&self) -> Handle<Node> {
        // For the moment, return the first camera in the first model
        self.get_default_model().get_first_node_with_camera()
    }

    /// Returns the node with the default camera.
    pub fn get_default_camera_node(&self) -> &Node {
        // For the moment, return the first camera in the first model
        let hnode = self.get_default_camera_node_handle();
        self.get_default_model().get_node(hnode).unwrap()
    }

    /// Returns a mutable reference to the node with the default camera.
    pub fn get_default_camera_node_mut(&mut self) -> &mut Node {
        // For the moment, return the first camera in the first model
        let hnode = self.get_default_model().get_first_node_with_camera();
        self.get_default_model_mut().get_node_mut(hnode).unwrap()
    }

    /// Returns the default camera.
    pub fn get_default_camera(&self) -> &Camera {
        let node = self.get_default_camera_node();
        self.get_default_model()
            .get_camera(node.camera.unwrap())
            .unwrap()
    }

    /// Returns a mutable reference to the default camera.
    pub fn get_default_camera_mut(&mut self) -> &mut Camera {
        let hcamera = self.get_default_camera_node().camera.unwrap();
        self.get_default_model_mut()
            .get_camera_mut(hcamera)
            .unwrap()
    }

    /// Returns the default camera's draw info.
    pub fn get_default_camera_draw_info(&self) -> CameraDrawInfo {
        let hmodel = self.get_default_model_handle();
        let model = self.get_model(hmodel).unwrap();
        let hnode = model.get_first_node_with_camera();
        let node = model.get_node(hnode).unwrap();
        CameraDrawInfo {
            camera: node.camera.unwrap(),
            node: hnode,
            model: hmodel,
        }
    }
}
