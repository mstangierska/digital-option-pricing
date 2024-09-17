# Digital Option Pricer

# Project Specs

You are to build a Digital option pricer. The option can be a call or put. The option observation type can be American or European. For simplicity, we assume that the implied volatility of the underlying is the same for every maturity and strike, and rates are zero. The pricer will be coded in Rust.

1. Document yourself on the product and create a relevant Rust struct for the product and its features.
2. Evaluate two pricing methodologies: FEM and Monte Carlo.
    1. FEM: Finite element method. What are the pros and cons? Would you use adaptive meshing and, generally speaking, which variants would you consider? Elaborate.
    2. Monte-Carlo: What are the pros and cons? How would you go about generating the samples? What are the candidate variants of MC?
3. You decide to code the 2 pricing methods and leave it to the operator to decide. The final products will thus:
    - Get the market and product inputs relevant to the pricing.
    - Let the operator decide on the pricing method.
    - Output a price.

# Assumptions

Overall project assumptions, as per project specification, are as follows:

- Implied volatility, $v$, is constant, $\frac{dv}{dt} = 0$
- Interest-free rates, $r = 0$
- (Implied) dividend rates, $w = 0$

# Black-Scholes Formula

The Black-Scholes formula for pricing options can be written as follows:

$$
\frac{\partial V}{\partial t} + \frac{1}{2} \sigma^2 S^2 \frac{\partial^2 V}{\partial S^2} + r S \frac{\partial V}{\partial S} - r V = 0
$$

Where:
- $V$: Option value (underlying asset price)
- $S$: Current price of the underlying asset
- $t$: Time
- $r$: Risk-free interest rate
- $\sigma$: Volatility of the stock's returns (annualized standard deviation)

For $r = 0$ (as per our assumptions), we obtain:

$$
\frac{\partial V}{\partial t} + \frac{1}{2} \sigma^2 S^2 \frac{\partial^2 V}{\partial S^2} = 0
$$

Setting $\tau = T - t$, where $T$ is the expiration time, gives:

$$
\frac{\partial V}{\partial \tau} = \frac{1}{2} \sigma^2 S^2 \frac{\partial^2 V}{\partial S^2}
$$

# Finite Element (FE) Method

Advantages of the method [\textit{source}](http://www.juergentopper.de/pdfs/fe_wilmott.pdf):

- Solution for the entire domain is computed, as opposed to FDM (finite difference method), which focuses on isolated nodes.
- Can easily deal with high mesh curvature through, e.g., adaptive meshing.
- Measures of sensitivity (\textit{Greeks}) can be obtained more exactly with FE.
- FE can easily be adapted for the treatment of (semi-) infinite domains.

**Cons**:

- Computationally intensive.
- Sensitive to assumptions and inputs.

## Relevant Equations

As described in [\textit{source}], the Finite Element Method (with mass lumping) and Finite Difference centered scheme are equivalent if the mesh is uniform (although they have not been obtained in the same manner).

Using the finite difference equation, the equations can be discretized, and the Greeks are calculated. Following this process, the option price is interpolated between nodes at a given time.

If the American option type is chosen, a maximum of the payoff vs. immediate exercise of an option is calculated.

## Adaptive Meshing

Adaptive meshing can significantly improve computational efficiency and accuracy by varying the resolution in different parts of the lattice. The method is beneficial for options with features that create highly nonlinear value regions [\textit{source}](https://www.sciencedirect.com/science/article/pii/S0304405X99000240) and around the strike price at expiration for vanilla options. **Due to time constraints for this work, the adaptive meshing is not implemented**.

# Monte Carlo Method

Monte Carlo methods use random evaluations to approximate a quantity of interest. Monte Carlo sampling concerns sampling from distribution Q with limited knowledge about the distribution [\textit{source}](https://faculty.washington.edu/yenchic/19A_stat535/Lec9_MC.pdf).

**Advantages**:

- Easily extendable to exotic options.
- Easier to implement than FEM.

**Cons**:

- Computationally intensive.
- Sensitive to assumptions and inputs.
- Black swan events cannot be modeled due to implied volatility assumptions.

## MC Equations

The asset price at time $T$ can be expressed as follows:

$$
S_T = S_0 e^{\left( r - \frac{\sigma^2}{2} \right) T + \sigma W_T}
$$

Where $W_T$ follows the normal distribution with mean 0 and variance $T$.

Given $r = 0$, this simplifies to give:

$$
S_T = S_0 e^{\left(- \frac{\sigma^2}{2} \right) T + \sigma W_T}
$$

The pay-off of the call option is $max(S_T - K,0)$, and for the put option, it is $max(K - S_T, 0)$, where $K$ is the strike price.

### American-type Monte Carlo Algorithm

For American-type observation, the Longstaff-Schwartz algorithm, also known as the Least Squares Monte-Carlo (LSM) algorithm, needs to be implemented, as described in *Valuing American Options by Simulation: A Simple Least-Squares Approach* by Longstaff *et. al.* [\textit{source}](https://people.math.ethz.ch/~hjfurrer/teaching/LongstaffSchwartzAmericanOptionsLeastSquareMonteCarlo.pdf).

## Generating MC Paths

Random behavior of prices can be estimated using a standard normal (Gaussian) distribution. When the time step of a random walk is made infinitesimally small, the random walk becomes a Brownian motion.

The standard Brownian motion in continuous time can be described using the following equation:

$$
dS = \mu dt + \sigma dX
$$

Where:
- $dS$: Change of the asset price in continuous time $dt$.
- $dX$: Random variable from a normal distribution.
- $\sigma$ (assumed constant): Price volatility.
- $\mu$: Growth rate of asset price.

The advantages of this sampling method include normality and the Markov ('no memory') property. While not done in this work, Geometric Brownian Motion can also be used [\textit{source}](https://www.simtrade.fr/blog_simtrade/brownian-motion-finance/).

## Variants of Monte Carlo Method

Variants of Monte Carlo include the Control Variate Method and Antithetic Variate Method, both of which were developed to reduce volatility in the pricing due to outliers. The conventional method was chosen for this simulation.

# Results

The results were validated against Omni's calculator Black-Scholes Simulation results [\textit{source}](https://www.omnicalculator.com/finance/black-scholes). Good agreement is noted (less than 1% difference), as shown in Table [ref:table] [\textit{source}](https://www.omnicalculator.com/finance/black-scholes).

| Strike | Spot Price | t  | Volatility | call/put | eu/us | fem/mc | Result of the Simulation | BS Calculator Online |
|--------|------------|----|------------|----------|-------|--------|--------------------------|----------------------|
| 100    | 100        | 1  | 15         | call     | eu    | fem    | 5.966                    | 5.98                 |
| 100    | 100        | 1  | 15         | call     | eu    | mc     | 5.966                    | 5.98                 |
| 100    | 200        | 1  | 15         | call     | eu    | mc     | 99.99                    | 100                  |
| 100    | 200        | 1  | 30         | call     | eu    | mc     | 99.97                    | 100.15               |
| 100    | 200        | 3  | 30         | put      | eu    | fem    | 6.89                     | 6.9                  |

*Table: Validation of results of simulation with online calculator. A good agreement (<1% difference) is observed.*
