use leptos::*;
use permaweb_lib::profile::{Profile, Zone, Wallet};

// Quantum Validator Visualizer Component
#[component]
pub fn QuantumValidator() -> impl IntoView {
    // ActorX implementation with Permaweb Profile
    let profile = Profile::new("IMRT-Visualizer");
    let zone = Zone::new(&profile);
    let wallet = Wallet::new(&profile);
    
    view! {
        <div class="quantum-validator">
            <h2>"Quantum Validator Status"</h2>
            <div class="validator-interface">
                // Validator interface components
            </div>
        </div>
    }
}

// JAM Visualizer Component
#[component]
pub fn JAMVisualizer() -> impl IntoView {
    // ActorX implementation with Permaweb Profile
    let profile = Profile::new("IMRT-JAM-Visualizer");
    let zone = Zone::new(&profile);
    let wallet = Wallet::new(&profile);
    
    view! {
        <div class="jam-visualizer">
            <h2>"JAM System Status"</h2>
            <div class="jam-interface">
                // JAM interface components
            </div>
        </div>
    }
}
