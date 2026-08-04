use core::mem;

use crate::backend::Backend;
use crate::layout::{Position, Rect, Size};
use crate::terminal::{Terminal, Viewport};

impl<B: Backend> Terminal<B> {
    /// Sets the height of an inline viewport and resizes it accordingly.
    ///
    /// This method only works with inline viewports. For other viewport types, it has no effect.
    /// The viewport will be resized to the new height, and the buffers will be cleared and
    /// reallocated to match the new size.
    ///
    /// # Arguments
    ///
    /// * `new_height` - The new height for the inline viewport in lines
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use ratatui::{Terminal, TerminalOptions, Viewport};
    ///
    /// let mut terminal = Terminal::with_options(backend, TerminalOptions {
    ///     viewport: Viewport::Inline(8),
    /// })?;
    ///
    /// // Later, resize the viewport to 12 lines
    /// terminal.set_viewport_height(12)?;
    /// ```
    pub fn set_viewport_height(&mut self, new_height: u16) -> Result<(), B::Error> {
        let Viewport::Inline(height) = &mut self.viewport else {
            return Ok(());
        };
        if *height == new_height {
            return Ok(());
        }

        let old_height = mem::replace(height, new_height);
        if new_height > old_height {
            let diff = new_height - old_height;
            self.set_cursor_position(Position::new(0, old_height.saturating_sub(1)))?;
            self.backend.append_lines(diff)?;
            self.backend.move_cursor_relative(0, -(diff as i16))?;
            self.backend
                .move_cursor_relative(-(self.viewport_area.width as i16), 0)?;
            self.inline_cursor_x = 0;
            self.inline_cursor_y = old_height.saturating_sub(1);
        }

        self.set_viewport_area(Rect {
            height: new_height,
            y: 0,
            ..self.viewport_area
        });

        // Assume every cell should be rewritten as we might have scrolled up
        self.buffers[1 - self.current].reset();
        Ok(())
    }
}

/// Compute the on-screen area for an inline viewport.
///
/// This helper is used by [`Terminal::with_options`] (initialization) and [`Terminal::resize`]
/// (after a terminal resize) to translate `Viewport::Inline(height)` into a concrete [`Rect`].
pub(crate) fn compute_inline_size<B: Backend>(
    backend: &mut B,
    height: u16,
    size: Size,
) -> Result<(Rect, Position), B::Error> {
    let max_height = size.height.min(height);

    backend.move_cursor_relative(-(size.width as i16), 0)?;

    if max_height > 1 {
        let lines_to_append = max_height - 1;
        backend.append_lines(lines_to_append)?;
        backend.move_cursor_relative(0, -(lines_to_append as i16))?;
        backend.move_cursor_relative(-(size.width as i16), 0)?;
    }

    Ok((
        Rect {
            x: 0,
            y: 0,
            width: size.width,
            height: max_height,
        },
        Position::ORIGIN,
    ))
}

#[cfg(test)]
mod tests {
    use crate::backend::{Backend, TestBackend};
    use crate::layout::{Position, Rect, Size};
    use crate::terminal::inline::compute_inline_size;

    #[test]
    fn compute_inline_size_preallocates_and_moves_cursor_back_relative() {
        let mut backend = TestBackend::new(10, 10);
        let (area, observed_pos) = compute_inline_size(&mut backend, 4, Size::new(10, 10)).unwrap();

        assert_eq!(observed_pos, Position { x: 0, y: 0 });
        assert_eq!(area, Rect::new(0, 0, 10, 4));
    }

    #[test]
    fn compute_inline_size_clamps_height_to_terminal_size() {
        let mut backend = TestBackend::new(10, 5);
        let (area, observed_pos) = compute_inline_size(&mut backend, 10, Size::new(10, 5)).unwrap();

        assert_eq!(observed_pos, Position { x: 0, y: 0 });
        assert_eq!(area, Rect::new(0, 0, 10, 5));
    }
}
