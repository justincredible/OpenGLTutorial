module Frame (Frame,Frame.initialize) where

import Control.Monad
import Data.Foldable
import Graphics.GL
import Graphics.UI.GLFW

import Camera
import Flow.Parameters
import Flow.Render
import Flow.Shutdown
import Flow.Update
import Light
import Maths
import Model
import OpenGL
import SpecularMapShader

data Frame = Frame {
    getWindow :: Window,
    getOpenGL :: OpenGL,
    getDemoRot :: GLfloat,
    getCamera :: Camera,
    getLight :: Light,
    getModel :: Maybe Model,
    getShader :: Maybe SpecularMapShader }
    deriving (Eq, Show)

initialize window width height = do
    opengl <- OpenGL.initialize window width height
    camera <- fmap snd $ Camera.initialize >>= render
    light <- Light.initialize
    (modelscs,model) <- Model.initialize "asset/cube.txt" ["asset/stone02.tga","asset/bump02.tga","asset/spec02.tga"] 0 False
    (shaderscs,shader) <- SpecularMapShader.initialize
    return (modelscs && shaderscs, Just $
        Frame window opengl 0 camera light model shader)

instance Render Frame where
    render frame = do
        beginScene 0 0 0 1
        
        (_,camera) <- render (getCamera frame)
        
        let Just model = getModel frame
            Just shader = getShader frame
            light = getLight frame
        
        parameters shader
            (yRotationLH . getDemoRot $ frame)
            (getView camera)
            (getProjection . getOpenGL $ frame)
            (getPosition camera)
            (getDirection light)
            (getDiffuse light)
            (getSpecular light)
            (getPower light)
            (modelTexture model)
            
        render model
        
        swapBuffers . getWindow $ frame
        
        let rotation = (+ pi*0.0025) . getDemoRot $ frame
        return . (,) True $ frame { getCamera = camera, getDemoRot = if rotation > 2*pi then rotation - 2*pi else rotation }

instance Update Frame where
    update frame None = return (True, frame { getCamera = (getCamera frame) { getPosition = [0,0,-5] }})
    update frame _ = do
        putStrLn "Incorrect frame parameters."
        return (False,frame)

instance Shutdown Frame where
    shutdown frame = do
        shutdown . getModel $ frame
        shutdown . getShader $ frame
