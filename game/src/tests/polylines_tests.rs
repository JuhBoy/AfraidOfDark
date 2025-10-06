#[cfg(test)]
mod tests {
    use crate::engine::utils::maths::Grid;

    #[test]
    fn grid_should_have_the_correct_length_when_using_full_grid_mode() {
        let grid: Grid = Grid::new(10, 10, 2f32);

        let w_len = grid.columns.len();
        let h_len: usize = grid.rows.len();

        let iter_chain = grid.columns.iter().chain(grid.rows.iter());
        let points_count = iter_chain.fold(0, |mut sum, p| {
            sum += p.points.len();
            sum
        });

        assert_eq!(w_len, 10);
        assert_eq!(h_len, 10);
        assert_eq!(grid.len(), 20);
        assert_eq!(points_count, 200);
    }
}
