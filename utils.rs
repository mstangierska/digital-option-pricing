use rand_distr::StandardNormal;
use rand::Rng;
use ndarray::{Array2, stack, indices, prelude::*};

//matrix generation function
pub fn generate_matrix(n: usize, m: usize) -> Array2<f64> {
    let mut rng = rand::thread_rng();
    let data: Vec<f64> = (0..(n * m)).map(|_| rng.sample(StandardNormal)).collect();
    Array2::from_shape_vec((n, m), data).unwrap()
}

//scales by implied volatility and number of trading days
pub fn scale_paths(matrix: &mut Array2<f64>, volatility: f64) {
    let trading_days: f64 = 252.0;
    let adjusted_volatility = volatility / trading_days.sqrt();
    
    for element in matrix.iter_mut() {
        *element *= adjusted_volatility;
    }
}

//generates a cumulative product matrix
pub fn cumulative_prod(matrix: Array2<f64>, spot_price: f64) -> Array2<f64> {
    let adjusted_matrix = matrix.mapv(|val| val + 1.0);
    let mut cumprod_matrix: ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 2]>> = Array2::zeros(adjusted_matrix.dim());

    for ((i, j), &val) in adjusted_matrix.indexed_iter() {
        if j == 0 {
            cumprod_matrix[[i, j]] = val;
        } else {
            cumprod_matrix[[i, j]] = cumprod_matrix[[i, j - 1]] * val;
        }
    }

    cumprod_matrix.mapv_inplace(|val| val * spot_price);
    return cumprod_matrix
}

