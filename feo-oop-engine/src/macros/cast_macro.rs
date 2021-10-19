//! Casting in rust can be a bit ugly

#[macro_export]
macro_rules! downcast {
    
    ($a:expr, dyn Global, $t:ty) => {
        {
            $a.as_any().downcast_ref::<$t>().unwrap().clone()
        }
    };

    // unsafe
    ($a:expr, Arc<$(RwLock<)?dyn GameObject$(>)+, Arc<$t:ty>) => { 
        {
            let this = Arc::into_raw($a).cast::<$t>();
            unsafe {Arc::from_raw(this)}
        }
    };
    ($a:expr, Arc<$(RwLock<)?dyn Camera$(>)?>, Arc<$t:ty>) => {
        {
            let this = Arc::into_raw($a).cast::<$t>();
            unsafe {Arc::from_raw(this)}
        }
    };
    ($a:expr, Arc<$(RwLock<)?dyn Light$(>)?>, Arc<$t:ty>) => {
        {
            let this = Arc::into_raw($a).cast::<$t>();
            unsafe {Arc::from_raw(this)}
        }
    };
    ($a:expr, Arc<$(RwLock<)?dyn Any$(>)?>, Arc<$t:ty>) => {
        {
            let this = Arc::into_raw($a).cast::<$t>();
            unsafe {Arc::from_raw(this)}
        }
    };
}

#[macro_export]
macro_rules! upcast {
    // TODO: warn do manually
    ($a:expr, Arc<RwLock<dyn Camera>>, Arc<RwLock<dyn GameObject>>) => {
        $a.read().unwrap().cast_gameobject_arc_rwlock($a);
    };
    ($a:expr, Arc<RwLock<dyn Light>>, Arc<RwLock<dyn GameObject>>) => {
        $a.read().unwrap().cast_gameobject_arc_rwlock($a);
    };
    ($a:expr, Arc<RwLock<$t:ty>>, Arc<RwLock<dyn GameObject>>) => { 
        { $a as Arc<RwLock<dyn GameObject>> }
    };
    ($a:expr, Arc<RwLock<$t:ty>>, Arc<RwLock<dyn Camera>>) => {
        { $a as Arc<RwLock<dyn Camera>> }
    };
    ($a:expr, Arc<RwLock<$t:ty>>, Arc<RwLock<dyn Light>>) => {
        { $a as Arc<RwLock<dyn Light>> }
    };

    ($a:expr, Arc<RwLock<$t:ty>>, Arc<dyn GameObject>) => { 
        { $a as Arc<dyn GameObject> }
    };
    ($a:expr, Arc<$t:ty>, Arc<dyn Camera>) => { 
        { $a as Arc<dyn Camera> }
    };
    ($a:expr, Arc<$t:ty>, Arc<dyn Light>) => { 
        { $a as Arc<dyn Light> }
    };
}