module Object (loadObj) where

loadObj :: FilePath -> IO ([[Float]],[[Float]],[[Float]])
loadObj file = do
    (vertices,uvcoords,normals,indices) <- fmap (compile id id id id.filter include.lines) $ readFile file
    
    return . process vertices uvcoords normals id id id $ indices
    where
    include ('v':_) = True
    include ('f':_) = True
    include _ = False
    compile dv dt dn di [] = (dv [], dt [], dn [], di [])
    compile dv dt dn di (x:xs)
        | take 2 x == "v " = let vertex = map read . tail . split ' ' $ x in
            compile (dv.(vertex:)) dt dn di xs
        | take 2 x == "vt" = let [u,v] = map read . tail . split ' ' $ x in
            compile dv (dt.([u,-v]:)) dn di xs
        | take 2 x == "vn" = let normal = map read . tail . split ' ' $ x in
            compile dv dt (dn.(normal:)) di xs
        | take 2 x == "f " = let [i1,i2,i3] = map (map read.split '/') . tail . split ' ' $ x in
            compile dv dt dn (di.(i1:).(i2:).(i3:)) xs
        | otherwise = error "Incorrect file format."
    split c s = let (chunk, rest) = break (== c) s in
        case rest of
            [] -> [chunk]
            _:rest -> chunk : split c rest
    process xs ys zs dx dy dz [] = (dx [], dy [], dz [])
    process xs ys zs dx dy dz ([x,y,z]:indices) =
        process xs ys zs
            (dx.((head . drop (x-1)) xs:))
            (dy.((head . drop (y-1)) ys:))
            (dz.((head . drop (z-1)) zs:))
            indices
