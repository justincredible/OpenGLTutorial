module LightMapShader (LightMapShader,initialize,parameters) where

import Foreign.C.String
import Foreign.Marshal.Array
import Foreign.Marshal.Utils
import Graphics.GL

import Flow.Render
import Flow.Shutdown
import ShaderCompilinker

data LightMapShader = LightMapShader {
    getProgram :: GLuint,
    getShaders :: [GLuint],
    getWorldLocation :: GLint,
    getViewLocation :: GLint,
    getProjectionLocation :: GLint,
    getTextureLocation :: GLint }
    deriving (Eq, Show)

initialize = do
    (success, program, shaders) <- compileAndLink ["glsl/lightmap.vert", "glsl/lightmap.frag"]
    
    if not success
    then return (False, Nothing)
    else do
        world <- withArray0 0 (map castCharToCChar "world") $ glGetUniformLocation program
        view <- withArray0 0 (map castCharToCChar "view") $ glGetUniformLocation program
        projection <- withArray0 0 (map castCharToCChar "projection") $ glGetUniformLocation program
        texlocn <- withArray0 0 (map castCharToCChar "texas") $ glGetUniformLocation program
        
        let success = all (/= -1) [world,view,projection,texlocn]
        return (success, Just $ LightMapShader
            program shaders world view projection texlocn)

instance Shutdown LightMapShader where
    shutdown (LightMapShader program shaders _ _ _ _) = do
        sequence_ $ map (glDetachShader program) shaders
        
        sequence_ $ map glDeleteShader shaders
        
        glDeleteProgram program

parameters (LightMapShader program _ world view projection texlocn)
    worldMatrix viewMatrix projectionMatrix texunit = do
        glUseProgram program

        withArray worldMatrix $ glUniformMatrix4fv world 1 GL_FALSE

        withArray viewMatrix $ glUniformMatrix4fv view 1 GL_FALSE

        withArray projectionMatrix $ glUniformMatrix4fv projection 1 GL_FALSE

        glUniform1i texlocn texunit
