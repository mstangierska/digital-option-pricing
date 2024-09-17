use wasm_bindgen::prelude::*;
use rand_distr::StandardNormal;
use rand::Rng;
use ndarray::{Array2, stack, indices, prelude::*, Zip};
use rusty_machine::learning::lin_reg::LinRegressor;
use rusty_machine::learning::SupModel;
use rusty_machine::linalg::{Matrix, Vector};
use crate::utils::{generate_matrix, scale_paths, cumulative_prod};

mod utils;

fn mc_european(matrix: Array2<f64>, strike: f64, q: f64) -> f64 {
    // get last value of each row
    let last_elements: Array1<f64> = matrix.axis_iter(Axis(0))
        .map(|row| *row.last().unwrap())
        .collect();
    
    // Set elements to 0 if smaller than strike
    let adjusted_elements: Array1<f64> = last_elements.mapv(|element| {
        let mut adjusted = q * (element - strike);
        if adjusted < 0.0 {
            adjusted = 0.0;
        }
        return adjusted
    });

    //calculate average value
    let sum: f64 = adjusted_elements.iter().sum();
    let count: f64 = adjusted_elements.len() as f64;
    let average = sum / count as f64;
    println!("Average value: {}", average);

    return average
}

fn mc_american(matrix: Array2<f64>, strike: f64, q: f64, m: usize, n: usize) -> f64 {
    //create and fill cash_flow array
    let mut cash_flow: Array2<f64> = Array2::zeros((n,m));
    for i in 1..n {
        for k in 1..m {
            cash_flow[[i, k]] = f64::max(q*(matrix[[i,k]] - strike), 0.0);
        }
    }


    for t in 1..n {
        //identify where the value of subarray's column t is larger than strike price; store in boolean array in_money
        let subarray: Array1<f64> = matrix.index_axis(Axis(1), t).to_owned();
        let in_money = subarray.mapv(|x| x < strike);

        //filter elements by in_money
        let filtered_elements: Vec<f64> = subarray
        .iter()
        .zip(in_money.iter())
        .filter_map(|(&value, &is_in_money)| if is_in_money { Some(value) } else { None })
        .collect();

        //get arrays of filtered elements and squared filtered elements
        let filtered_elements_array: Array1<f64> = Array1::from(filtered_elements);
        let filtered_elements_array_squared = filtered_elements_array.mapv(|x| x.powi(2));
    
        //stack columns
        let x: Array2<f64> = stack![
            Axis(1),
            filtered_elements_array.view(),
            filtered_elements_array_squared.view()
        ];

        //create vector and a matrix for training
        let x_vec: Vec<f64> = x.iter().cloned().collect();
        let x_matrix: Matrix<_> = Matrix::new(filtered_elements_array.len(), 2, x_vec);

        //get array of values in column t+1, where rows match those selected by in_money
        let subarray_cash: Array1<f64> = cash_flow.index_axis(Axis(1), t-1).to_owned();
        let y: Vec<f64> = subarray_cash
        .iter()
        .zip(in_money.iter())
        .filter_map(|(&value, &is_in_money)| if is_in_money { Some(value) } else { None })
        .collect();
        
        //create y vector for training
        let y_vector = Vector::new(y);

        // perform linear regression
        let mut linearRegression = LinRegressor::default(); 
        linearRegression.train(&x_matrix, &y_vector);

        let preds = linearRegression.predict(&x_matrix).unwrap();
        println!("y: {:?}", preds);
        println!("y_true: {:?}", y_vector);

        let mut continuations = Array1::zeros(filtered_elements.len());

        //basially trying to implement https://medium.com/@ptlabadie/pricing-american-options-in-python-8e357221d2a9

    }

    return m as f64
}


fn monte_carlo(spot_price: f64, strike: f64, n: usize, m: usize, volatility: f64, p_type: &str, t: f64, observation_type: &str) -> f64 {
    //get matrix of 0s
    let mut matrix = generate_matrix(n,m);
    scale_paths(&mut matrix, volatility);

    //get matrix of paths
    let paths: Array2<f64> = cumulative_prod(matrix, spot_price);

    //initialise result
    let mut result = 0.0;

    //distinguish between 'put' and 'call'
    let mut q:f64= 1.0;
    if p_type == "put" {
        q = -1.0;
    }

    //obtain result, depending on observation type
    if observation_type == "european" {
        result = mc_european(paths, strike, q);
    }
    else {
        result = mc_american(paths, strike, q, m, n);
    }

    return result
}


fn main() {
    let strike = 100.0;
    let spot_price: f64 = 100.0;
    let n = 10; // number of runs
    let m = 252; // number of days in trading year: 365 is 1.0
    let volatility: f64 = 0.15; //we assume it does not change
    let p_type = "put"; //put or call
    let t = 1.0; //time horizon
    let observation_type = "american"; //american or european
    let calculation_method = "mc"; //select calculation method

    //we always begin the same way
    if calculation_method == "mc"
    {
        let result = monte_carlo(spot_price, strike, n, m, volatility, p_type, t, observation_type);
    }

    //println!("{:?}", last_elements);

    //FEM - TBC
    // let p_type = "put";
    // let nas: usize = 10;
    // let t = 1.0;
    // let (finite, locations) = finite_difference(volatility, p_type, strike, t, nas);
    // println!("{:?}", finite);
    //println!("locations: {:?}", locations);

}

fn finite_difference(volatility: f64, p_type: &str, strike: f64, t: f64, nas: usize) -> (Array2<f64>, Vec<(usize, usize)>) {
    let nts: usize = 1000; //number of time steps
    let dt: f64 = t / (nts as f64); //time step
    let ds = 2.0 * strike / (nas as f64); //change in price  
    let s: Vec<f64> = (0..=nas).map(|i| i as f64 * ds).collect();
    let mut v: Array2<f64> = Array2::zeros((nas + 1, nts + 1));
    let mut payoff: ArrayBase<ndarray::OwnedRepr<f64>, Dim<[usize; 1]>> = Array1::zeros(nas+1);
    let mut locations: Vec<(usize, usize)> = Vec::new();

    // println!("s {:?}", s);

    let mut q = 1;
    if p_type == "put" {
        q = -1;
    }

    for i in 1..nas+1 {
        let payoff_value = f64::max((q as f64) * (s[i] - strike), 0.0);
        v[[i, 0]] = payoff_value;
        payoff[i] = payoff_value;
    }

    // println!("payoff {:?}", payoff);

    
    for k in 1..nts+1 {
        // Asset loop
        for i in 1..nas {
            // let delta = (v[[i + 1, k - 1]] - v[[i - 1, k - 1]]) / (2.0 * ds); // Central difference
            let gamma = (v[[i + 1, k - 1]] - 2.0 * v[[i, k - 1]] + v[[i - 1, k - 1]]) / (ds * ds); // Central difference
            let theta = -0.5 * volatility.powi(2) * s[i].powi(2) * gamma;
            v[[i, k]] = v[[i, k - 1]] - dt * theta;
            if payoff[i] > v[[i,k]]{
                let pair = (i,k);
                locations.push(pair);
            }         
        }

        // Boundary condition at S=0
        v[[0, k]] = v[[0, k - 1]] * (1.0);

        // Boundary condition at S=infinity
        v[[nas, k]] = 2.0 * v[[nas - 1, k]] - v[[nas - 2, k]];
    }


    return(v, locations)

}

