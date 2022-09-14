module Maths where

{- Maths:
    Linear Algebra functions for vectors and matrices
-}

import Graphics.GL

import Data.List

add :: Num a => [a] -> [a] -> [a]
add = zipWith (+)

minus a b = add a $ map negate b

multiply3 [x,y,z] [[m11,m21,m31],[m12,m22,m32],[m13,m23,m33]] =
    [ x*m11 + y*m12 + z*m13, x*m21 + y*m22 + z*m23, x*m31 + y*m32 + z*m33 ]

matscale4 s = map (map (*s))
matmult4 a b = ($ []) . (flip (flip foldl' id)) b $ \dl bs -> (dl.).(:) $ [
    foldr (+) 0 . zipWith (*) (map head a) $ bs,
    foldr (+) 0 . zipWith (*) (map (head.tail) $ a) $ bs,
    foldr (+) 0 . zipWith (*) (map (head.tail.tail) $ a) $ bs,
    foldr (+) 0 . zipWith (*) (map (head.tail.tail.tail) $ a) $ bs ]

normalize :: Floating a => [a] -> [a]
normalize = map . flip (/) . sqrt . foldr (+) 0 . map (**2) <*> id

-- Right-Hand cross
cross [x,y,z] [x',y',z'] = [ y*z' - z*y', z*x' - x*z', x*y' - y*x' ]

dot :: Num a => [a] -> [a] -> a
dot = (foldr (+) 0 .) . zipWith (*)

buildModelViewPerspective fov aspect near far position lookat up scale =
    matmult4 (perspective4 fov aspect near far) $
        matmult4 (view4 position lookat up) (scale4 scale scale scale)
    where
    identity4 = [[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]]
    scale4 x y z = [[x,0,0,0],[0,y,0,0],[0,0,z,0],[0,0,0,1]]
    translate4 x y z = [[1,0,0,0],[0,1,0,0],[0,0,1,0],[x,y,z,1]]
    view4 position lookat up = let
        zAxis@[zax,zay,zaz] = (normalize . minus lookat) position
        xAxis@[xax,xay,xaz] = (normalize . cross zAxis) up
        yAxis@[yax,yay,yaz] = cross xAxis zAxis
        in
        [ [xax, yax, zax, 0]
        , [xay, yay, zay, 0]
        , [xaz, yaz, zaz, 0]
        , [negate . dot xAxis $ position, negate . dot yAxis $ position, negate . dot zAxis $ position, 1 ] ]
    perspective4 fieldOfView aspect near far = let
        rtfov = recip . tan . (*0.5) $ fieldOfView
        denom = far - near
        in
        [ [rtfov*recip aspect, 0, 0, 0]
        , [0, rtfov, 0, 0]
        , [0, 0, (far+near)/denom, 1]
        , [0, 0, negate $ far*near/denom, 0 ] ]
