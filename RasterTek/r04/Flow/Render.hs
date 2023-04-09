module Flow.Render where

class Render a where
    render :: a -> IO (Bool,a)
    
instance Render a => Render (Maybe a) where
    render = maybe (pure (True,Nothing)) (fmap (fmap Just) . render)
