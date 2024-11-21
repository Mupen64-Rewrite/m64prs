macro_rules! empty_dispatch {
    (impl Dispatch<$object:ty, $data:ty> for $state:ty) => {
        impl ::wayland_client::Dispatch<$object, $data> for $state {
            fn event(
                _: &mut Self,
                _: &$object,
                _: <$object as ::wayland_client::Proxy>::Event,
                _: &$data,
                _: &::wayland_client::Connection,
                _: &::wayland_client::QueueHandle<Self>,
            ) {
            }
        }
    };
}

pub(crate) use empty_dispatch;
