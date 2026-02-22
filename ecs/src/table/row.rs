use crate::{AnyStorage, Column, EntityHandle};
use std::any::Any;

// src/table/row

/// Errors that can occur when manipulating a row.
#[derive(Debug)]
pub enum RowError {
    InvalidColumnIndex { index: usize },
    TypeMismatch { index: usize },
    SwapRemoveOutOfBounds { index: usize },
}
/// Represents a single row in an archetype.
pub struct Row<'a> {
    columns: &'a mut [Box<dyn AnyStorage>],
    row_index: usize,
    entity_column: &'a mut Column<EntityHandle>,
}
impl<'a> Row<'a> {
    /// Creates a new Row for the given columns, entity column, and row index
    pub fn new(
        columns: &'a mut [Box<dyn AnyStorage>],
        entity_column: &'a mut Column<EntityHandle>,
        row_index: usize,
    ) -> Self {
        Self {
            columns,
            row_index,
            entity_column,
        }
    }

    /// Returns the number of components (columns) in this row.
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Returns true if the row has no components.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Sets a component value at a column index.
    pub fn set<T: 'static>(&mut self, column_index: usize, value: T) -> Result<(), RowError> {
        let column = self
            .columns
            .get_mut(column_index)
            .ok_or(RowError::InvalidColumnIndex {
                index: column_index,
            })?;

        let typed_col =
            column
                .as_any_mut()
                .downcast_mut::<Column<T>>()
                .ok_or(RowError::TypeMismatch {
                    index: column_index,
                })?;

        typed_col.push(value);
        Ok(())
    }

    pub fn get<T: 'static>(&self, column_index: usize) -> Result<&T, RowError> {
        let column = self
            .columns
            .get(column_index)
            .ok_or(RowError::InvalidColumnIndex {
                index: column_index,
            })?;

        let typed_col =
            column
                .as_any()
                .downcast_ref::<Column<T>>()
                .ok_or(RowError::TypeMismatch {
                    index: column_index,
                })?;

        typed_col
            .get(self.row_index)
            .ok_or(RowError::SwapRemoveOutOfBounds {
                index: column_index,
            })
    }

    pub fn get_mut<T: 'static>(&mut self, column_index: usize) -> Result<&mut T, RowError> {
        let column = self
            .columns
            .get_mut(column_index)
            .ok_or(RowError::InvalidColumnIndex {
                index: column_index,
            })?;

        let typed_col =
            column
                .as_any_mut()
                .downcast_mut::<Column<T>>()
                .ok_or(RowError::TypeMismatch {
                    index: column_index,
                })?;

        typed_col
            .get_mut(self.row_index)
            .ok_or(RowError::SwapRemoveOutOfBounds {
                index: column_index,
            })
    }

    pub fn swap_remove_column(&mut self, column_index: usize) -> Result<Box<dyn Any>, RowError> {
        let column = self
            .columns
            .get_mut(column_index)
            .ok_or(RowError::InvalidColumnIndex {
                index: column_index,
            })?;

        column
            .swap_remove(self.row_index)
            .ok_or(RowError::SwapRemoveOutOfBounds {
                index: column_index,
            })
    }

    /// Swap-removes the entire row across all columns and the entity column.
    /// Returns the removed entity and a `Vec` of removed components.
    pub fn swap_remove_row(&mut self) -> Result<(EntityHandle, Vec<Box<dyn Any>>), RowError> {
        // Remove the entity first
        let entity = self.entity_column.swap_remove(self.row_index).ok_or(
            RowError::SwapRemoveOutOfBounds {
                index: self.row_index,
            },
        )?;

        // Remove each column at the same row index
        let mut removed_components = Vec::with_capacity(self.columns.len());
        for (i, col) in self.columns.iter_mut().enumerate() {
            removed_components.push(
                col.swap_remove(self.row_index)
                    .ok_or(RowError::SwapRemoveOutOfBounds { index: i })?,
            );
        }

        Ok((entity, removed_components))
    }
}
