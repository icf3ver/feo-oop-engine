//! Vulkano addon for three initial buffers

use std::marker::PhantomData;
use std::mem;
use std::sync::Arc;
use std::vec::IntoIter as VecIntoIter;

use vulkano::buffer::BufferAccess;
use vulkano::buffer::TypedBufferAccess;
use vulkano::pipeline::shader::ShaderInterfaceDef;
use vulkano::pipeline::vertex::AttributeInfo;
use vulkano::pipeline::vertex::IncompatibleVertexDefinitionError;
use vulkano::pipeline::vertex::InputRate;
use vulkano::pipeline::vertex::Vertex;
use vulkano::pipeline::vertex::VertexDefinition;
use vulkano::pipeline::vertex::VertexSource;

/// Unstable
pub(crate) struct ThreeBuffersDefinition<T, U, I>(pub PhantomData<(T, U, I)>);

impl<T, U, I> ThreeBuffersDefinition<T, U, I> {
    #[inline]
    pub fn new() -> ThreeBuffersDefinition<T, U, I> {
        ThreeBuffersDefinition(PhantomData)
    }
}

unsafe impl<T, U, I, J> VertexDefinition<J> for ThreeBuffersDefinition<T, U, I>
where
    T: Vertex,
    U: Vertex,
    I: Vertex,
    J: ShaderInterfaceDef,
{
    type BuffersIter = VecIntoIter<(u32, usize, InputRate)>;
    type AttribsIter = VecIntoIter<(u32, u32, AttributeInfo)>;

    fn definition(
        &self,
        interface: &J,
    ) -> Result<(Self::BuffersIter, Self::AttribsIter), IncompatibleVertexDefinitionError> {
        let attrib = {
            let mut attribs = Vec::with_capacity(interface.elements().len());
            for e in interface.elements() {
                let name = e.name.as_ref().unwrap();

                let (infos, buf_offset) = if let Some(infos) = <T as Vertex>::member(name) {
                    (infos, 0)
                } else if let Some(infos) = <U as Vertex>::member(name) {
                    (infos, 1)
                }  else if let Some(infos) = <I as Vertex>::member(name) {
                    (infos, 2)
                } else {
                    return Err(IncompatibleVertexDefinitionError::MissingAttribute {
                        attribute: name.clone().into_owned(),
                    });
                };

                if !infos.ty.matches(
                    infos.array_size,
                    e.format,
                    e.location.end - e.location.start,
                ) {
                    return Err(IncompatibleVertexDefinitionError::FormatMismatch {
                        attribute: name.clone().into_owned(),
                        shader: (e.format, (e.location.end - e.location.start) as usize),
                        definition: (infos.ty, infos.array_size),
                    });
                }

                let mut offset = infos.offset;
                for loc in e.location.clone() {
                    attribs.push((
                        loc,
                        buf_offset,
                        AttributeInfo {
                            offset,
                            format: e.format,
                        },
                    ));
                    offset += e.format.size().unwrap();
                }
            }
            attribs
        }
        .into_iter(); // TODO: meh

        let buffers = vec![
            (0, mem::size_of::<T>(), InputRate::Vertex),
            (1, mem::size_of::<U>(), InputRate::Vertex),
            (2, mem::size_of::<I>(), InputRate::Vertex),
        ]
        .into_iter();

        Ok((buffers, attrib))
    }
}

unsafe impl<T, U, I> VertexSource<Vec<Arc<dyn BufferAccess + Send + Sync>>>
    for ThreeBuffersDefinition<T, U, I>
where
    T: Vertex,
    U: Vertex,
    I: Vertex,
{
    #[inline]
    fn decode(
        &self,
        source: Vec<Arc<dyn BufferAccess + Send + Sync>>,
    ) -> (Vec<Box<dyn BufferAccess + Send + Sync>>, usize, usize) {
        // FIXME: safety
        assert_eq!(source.len(), 3);
        let vertices = [
            source[0].size() / mem::size_of::<T>(),
            source[1].size() / mem::size_of::<U>(),
            source[2].size() / mem::size_of::<I>(),
        ]
        .iter()
        .cloned()
        .min()
        .unwrap();
        (
            vec![Box::new(source[0].clone()), Box::new(source[1].clone()), Box::new(source[2].clone())],
            vertices,
            1,
        )
    }
}

unsafe impl<'a, T, U, I, Bt, Bu, Bi> VertexSource<(Bt, Bu, Bi)> for ThreeBuffersDefinition<T, U, I>
where
    T: Vertex,
    Bt: TypedBufferAccess<Content = [T]> + Send + Sync + 'static,
    U: Vertex,
    Bu: TypedBufferAccess<Content = [U]> + Send + Sync + 'static,
    I: Vertex,
    Bi: TypedBufferAccess<Content = [I]> + Send + Sync + 'static,
{
    #[inline]
    fn decode(&self, source: (Bt, Bu, Bi)) -> (Vec<Box<dyn BufferAccess + Send + Sync>>, usize, usize) {
        let vertices = [source.0.len(), source.1.len(), source.2.len()]
            .iter()
            .cloned()
            .min()
            .unwrap();
        (
            vec![Box::new(source.0) as Box<_>, Box::new(source.1) as Box<_>, Box::new(source.2) as Box<_>],
            vertices,
            1,
        )
    }
}
