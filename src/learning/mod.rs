mod domain;

pub use domain::GameDomain;

use rsrl::{
    control::td::QLearning,
    core::{make_shared, run, Evaluation, Parameter, SerialExperiment},
    domains::{CartPole, Domain},
    fa::{basis::fixed::Chebyshev, LFA},
    geometry::Space,
    logging,
    policies::fixed::{EpsilonGreedy, Greedy, Random},
};

pub struct Learning {}

impl Learning {
    pub fn learn() {
        let logger = logging::root(logging::stdout());

        let domain = GameDomain::default();
        let mut agent = {
            let n_actions = domain.action_space().card().into();

            // Build the linear value functions using a fourier basis projection.
            let bases = Chebyshev::from_space(1, domain.state_space());
            //let v_func = make_shared(LFA::scalar_output(bases.clone()));
            let q_func = make_shared(LFA::vector_output(bases, n_actions));

            // Build a stochastic behaviour policy with exponential epsilon.
            let policy = make_shared(EpsilonGreedy::new(
                Greedy::new(q_func.clone()),
                Random::new(n_actions),
                Parameter::exponential(0.3, 0.001, 0.99),
            ));

            QLearning::new(q_func, policy, 0.5, 0.5)
        };

        let mut c = 0;
        loop {
            c+=1;
            // Training phase:
            let _training_result = {
                // Start a serial learning experiment up to 1000 steps per episode.
                let e = SerialExperiment::new(&mut agent, Box::new(GameDomain::default), 1000);

                // Realise 1000 episodes of the experiment generator.
                run(e, 100, Some(logger.clone()))
            };

            // Testing phase:
            let testing_result = Evaluation::new(&mut agent, Box::new(GameDomain::default)).next().unwrap();

            info!(logger, "batch {}", c; agent.weights());
            info!(logger, "solution"; testing_result);
        }
    }
}
