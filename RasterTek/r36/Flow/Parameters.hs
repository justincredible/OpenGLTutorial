module Flow.Parameters (Parameters(..)) where

import Graphics.UI.GLFW

data Parameters = None
    | I Int
    | F Float
    | II Int Int
    | IF Int Float
    | LF [Float]
    deriving (Eq,Show)
