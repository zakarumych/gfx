// Copyright 2015 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Render target components for the PSO macro.

use std::marker::PhantomData;
use gfx_core::{ColorSlot, Resources};
use gfx_core::{format, handle, pso, state, target};
use gfx_core::factory::Typed;
use gfx_core::shade::OutputVar;
use super::{DataLink, DataBind, RawDataSet};

/// Render target component. Typically points to a color-formatted texture.
/// - init: `&str` = name of the target
/// - data: `RenderTargetView<T>`
pub struct RenderTarget<T>(Option<ColorSlot>, PhantomData<T>);
/// Render target component with active blending mode.
/// - init: (`&str`, `ColorMask`, `Blend` = blending state)
/// - data: `RenderTargetView<T>`
pub struct BlendTarget<T>(Option<ColorSlot>, PhantomData<T>);
/// Depth target component.
/// - init: `Depth` = depth state
/// - data: `DepthStencilView<T>`
pub struct DepthTarget<T>(PhantomData<T>);
/// Stencil target component.
/// - init: `Stencil` = stencil state
/// - data: (`DepthStencilView<T>`, `(front, back)` = stencil reference values)
pub struct StencilTarget<T>(PhantomData<T>);
/// Depth + stencil target component.
/// - init: (`Depth` = depth state, `Stencil` = stencil state)
/// - data: (`DepthStencilView<T>`, `(front, back)` = stencil reference values)
pub struct DepthStencilTarget<T>(PhantomData<T>);
/// Scissor component. Sets up the scissor test for rendering.
/// - init: `()`
/// - data: `Rect` = target area
pub struct Scissor(bool);
/// Blend reference component. Sets up the reference color for blending.
/// - init: `()`
/// - data: `ColorValue`
pub struct BlendRef;


impl<'a, T: format::RenderFormat> DataLink<'a> for RenderTarget<T> {
    type Init = &'a str;
    fn new() -> Self {
        RenderTarget(None, PhantomData)
    }
    fn is_active(&self) -> bool {
        self.0.is_some()
    }
    fn link_output(&mut self, out: &OutputVar, init: &Self::Init) ->
                   Option<Result<pso::ColorTargetDesc, format::Format>> {
        if out.name.is_empty() || &out.name == init {
            self.0 = Some(out.slot);
            let desc = (T::get_format(), state::MASK_ALL.into());
            Some(Ok(desc))
        }else {
            None
        }
    }
}

impl<R: Resources, T> DataBind<R> for RenderTarget<T> {
    type Data = handle::RenderTargetView<R, T>;
    fn bind_to(&self, out: &mut RawDataSet<R>, data: &Self::Data, man: &mut handle::Manager<R>) {
        if let Some(slot) = self.0 {
            out.pixel_targets.add_color(slot, man.ref_rtv(data.raw()), data.raw().get_dimensions());
        }
    }
}


impl<'a, T: format::BlendFormat> DataLink<'a> for BlendTarget<T> {
    type Init = (&'a str, state::ColorMask, state::Blend);
    fn new() -> Self {
        BlendTarget(None, PhantomData)
    }
    fn is_active(&self) -> bool {
        self.0.is_some()
    }
    fn link_output(&mut self, out: &OutputVar, init: &Self::Init) ->
                   Option<Result<pso::ColorTargetDesc, format::Format>> {
        if out.name.is_empty() || &out.name == init.0 {
            self.0 = Some(out.slot);
            let desc = (T::get_format(), pso::ColorInfo {
                mask: init.1,
                color: Some(init.2.color),
                alpha: Some(init.2.alpha),
            });
            Some(Ok(desc))
        }else {
            None
        }
    }
}

impl<R: Resources, T> DataBind<R> for BlendTarget<T> {
    type Data = handle::RenderTargetView<R, T>;
    fn bind_to(&self, out: &mut RawDataSet<R>, data: &Self::Data, man: &mut handle::Manager<R>) {
        if let Some(slot) = self.0 {
            out.pixel_targets.add_color(slot, man.ref_rtv(data.raw()), data.raw().get_dimensions());
        }
    }
}


impl<'a, T: format::DepthFormat> DataLink<'a> for DepthTarget<T> {
    type Init = state::Depth;
    fn new() -> Self { DepthTarget(PhantomData) }
    fn is_active(&self) -> bool { true }
    fn link_depth_stencil(&mut self, init: &Self::Init) -> Option<pso::DepthStencilDesc> {
        Some((T::get_format().0, (*init).into()))
    }
}

impl<R: Resources, T> DataBind<R> for DepthTarget<T> {
    type Data = handle::DepthStencilView<R, T>;
    fn bind_to(&self, out: &mut RawDataSet<R>, data: &Self::Data, man: &mut handle::Manager<R>) {
        let dsv = data.raw();
        out.pixel_targets.add_depth_stencil(man.ref_dsv(dsv), true, false, dsv.get_dimensions());
    }
}

impl<'a, T: format::StencilFormat> DataLink<'a> for StencilTarget<T> {
    type Init = state::Stencil;
    fn new() -> Self { StencilTarget(PhantomData) }
    fn is_active(&self) -> bool { true }
    fn link_depth_stencil(&mut self, init: &Self::Init) -> Option<pso::DepthStencilDesc> {
        Some((T::get_format().0, (*init).into()))
    }
}

impl<R: Resources, T> DataBind<R> for StencilTarget<T> {
    type Data = (handle::DepthStencilView<R, T>, (target::Stencil, target::Stencil));
    fn bind_to(&self, out: &mut RawDataSet<R>, data: &Self::Data, man: &mut handle::Manager<R>) {
        let dsv = data.0.raw();
        out.pixel_targets.add_depth_stencil(man.ref_dsv(dsv), false, true, dsv.get_dimensions());
        out.ref_values.stencil = data.1;
    }
}

impl<'a, T: format::DepthStencilFormat> DataLink<'a> for DepthStencilTarget<T> {
    type Init = (state::Depth, state::Stencil);
    fn new() -> Self { DepthStencilTarget(PhantomData) }
    fn is_active(&self) -> bool { true }
    fn link_depth_stencil(&mut self, init: &Self::Init) -> Option<pso::DepthStencilDesc> {
        Some((T::get_format().0, (*init).into()))
    }
}

impl<R: Resources, T> DataBind<R> for DepthStencilTarget<T> {
    type Data = (handle::DepthStencilView<R, T>, (target::Stencil, target::Stencil));
    fn bind_to(&self, out: &mut RawDataSet<R>, data: &Self::Data, man: &mut handle::Manager<R>) {
        let dsv = data.0.raw();
        out.pixel_targets.add_depth_stencil(man.ref_dsv(dsv), true, true, dsv.get_dimensions());
        out.ref_values.stencil = data.1;
    }
}


impl<'a> DataLink<'a> for Scissor {
    type Init = ();
    fn new() -> Self { Scissor(false) }
    fn is_active(&self) -> bool { self.0 }
    fn link_scissor(&mut self) -> bool { self.0 = true; true }
}

impl<R: Resources> DataBind<R> for Scissor {
    type Data = target::Rect;
    fn bind_to(&self, out: &mut RawDataSet<R>, data: &Self::Data, _: &mut handle::Manager<R>) {
        out.scissor = *data;
    }
}

impl<'a> DataLink<'a> for BlendRef {
    type Init = ();
    fn new() -> Self { BlendRef }
    fn is_active(&self) -> bool { true }
}

impl<R: Resources> DataBind<R> for BlendRef {
    type Data = target::ColorValue;
    fn bind_to(&self, out: &mut RawDataSet<R>, data: &Self::Data, _: &mut handle::Manager<R>) {
        out.ref_values.blend = *data;
    }
}
