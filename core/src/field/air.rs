use core::borrow::{Borrow, BorrowMut};
use core::mem::size_of;
use p3_air::{Air, AirBuilder, BaseAir};
use p3_field::AbstractField;
use p3_field::Field;
use p3_matrix::MatrixRowSlices;
use valida_derive::AlignedBorrow;

use crate::air::{CurtaAirBuilder, FieldAirBuilder};

use super::FieldLTUChip;

pub const LTU_NB_BITS: usize = 22;

pub const NUM_FIELD_COLS: usize = size_of::<FieldLTUCols<u8>>();

#[derive(Debug, Clone, Copy, AlignedBorrow)]
#[repr(C)]
pub struct FieldLTUCols<T> {
    /// The result of the `LT` operation on `a` and `b`
    pub lt: T,

    /// The first field operand.
    pub b: T,

    /// The second field operand.
    pub c: T,

    /// The difference between `b` and `c` in little-endian order.
    pub diff_bits: [T; LTU_NB_BITS + 1],

    // TODO:  Support multiplicities > 1.  Right now there can be duplicate rows.
    // pub multiplicities: T,
    pub is_real: T,
}

impl<F: Field> BaseAir<F> for FieldLTUChip {
    fn width(&self) -> usize {
        NUM_FIELD_COLS
    }
}

impl<AB: CurtaAirBuilder> Air<AB> for FieldLTUChip {
    fn eval(&self, builder: &mut AB) {
        let main = builder.main();
        let local: &FieldLTUCols<AB::Var> = main.row_slice(0).borrow();

        // Dummy constraint for normalizing to degree 3.
        builder.assert_eq(local.b * local.b * local.b, local.b * local.b * local.b);

        // Verify that lt is a boolean.
        builder.assert_bool(local.lt);

        // Verify that the diff bits are boolean.
        for i in 0..local.diff_bits.len() {
            builder.assert_bool(local.diff_bits[i]);
        }

        // Verify the decomposition of b - c.
        let mut diff = AB::Expr::zero();
        for i in 0..local.diff_bits.len() {
            diff += local.diff_bits[i] * AB::F::from_canonical_u32(1 << i);
        }
        builder.when(local.is_real).assert_eq(
            local.b - local.c + AB::F::from_canonical_u32(1 << LTU_NB_BITS),
            diff,
        );

        // Assert that the output is correct.
        builder
            .when(local.is_real)
            .assert_eq(local.lt, AB::Expr::one() - local.diff_bits[LTU_NB_BITS]);

        // Receive the field operation.
        builder.receive_field_op(local.lt, local.b, local.c, local.is_real);
    }
}