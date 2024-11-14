pub fn into_dpi_position(point: graphene::Point) -> dpi::LogicalPosition<f32> {
    dpi::LogicalPosition::new(point.x(), point.y())
}

pub fn into_graphene_point<P: dpi::Pixel>(point: dpi::LogicalPosition<P>) -> graphene::Point {
    graphene::Point::new(point.x.cast::<f32>(), point.y.cast::<f32>())
}

pub fn into_dpi_size(size: graphene::Size) -> dpi::LogicalSize<f32> {
    dpi::LogicalSize::new(size.width(), size.height())
}

pub fn into_graphene_size<P: dpi::Pixel>(size: dpi::LogicalSize<f32>) -> graphene::Size {
    graphene::Size::new(size.width, size.height)
}
