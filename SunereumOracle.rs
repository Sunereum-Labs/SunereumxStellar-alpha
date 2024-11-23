#![no_std]
use soroban_sdk::{
    contractimpl, symbol_short, Address, Env, String, Symbol, Vec, Map,
    contracttype,
};

#[derive(Clone)]
#[contracttype]
pub struct InverterData {
    timestamp: u64,
    energy_produced: u64,     // Wh for the hour
    peak_power: u32,          // W
    dc_voltage: u32,          // V
    dc_current: u32,          // mA
    ac_voltage: u32,          // V
    ac_frequency: u32,        // mHz
    internal_temp: i32,       // °C * 10
    efficiency: u32,          // percentage * 100
    power_factor: u32,        // percentage * 100
    daily_yield: u64,         // Wh
    total_yield: u64,         // kWh
    operating_hours: u32,     // hours
}

#[derive(Clone)]
#[contracttype]
pub struct Inverter {
    device_id: String,
    policy_id: String,
    last_ping: u64,
    operational_status: bool,
    online_status: bool,
    risk_level: String,
    device_type: String,      // "INVERTER" or other types
    manufacturer: String,
    model: String,
    rated_power: u32,         // W
    last_reading: InverterData,
    hourly_readings: Vec<InverterData>,
}

#[derive(Clone)]
#[contracttype]
pub struct EnergyMeter {
    meter_id: String,
    last_ping: u64,
    operational_status: bool,
    online_status: bool,
    manufacturer: String,
    model: String,
    last_reading: MeterData,
    hourly_readings: Vec<MeterData>,
}

#[derive(Clone)]
#[contracttype]
pub struct MeterData {
    timestamp: u64,
    energy_consumed: u64,     // Wh for the hour
    peak_power: u32,          // W
    voltage: u32,             // V
    current: u32,             // mA
    frequency: u32,           // mHz
    power_factor: u32,        // percentage * 100
    daily_consumption: u64,   // Wh
    total_consumption: u64,   // kWh
    operating_hours: u32,     // hours
}

#[derive(Clone)]
#[contracttype]
pub struct MaintenanceRecord {
    timestamp: u64,
    report_hash: String,
    technician: Address,
    status: String,
    maintenance_type: String,  // "SCHEDULED", "EMERGENCY", etc.
    issues_found: Vec<String>,
    parts_replaced: Vec<String>,
}

const DEVICES: Symbol = symbol_short!("DEVICES");
const MAINTENANCE: Symbol = symbol_short!("MAINTENANCE");
const DOWNTIME_THRESHOLD: u64 = 14400; // 4 hours in seconds
const AUTHORIZED_PROVIDERS: Symbol = symbol_short!("AUTH_PROVIDERS");
const PERFORMANCE_THRESHOLD: u32 = 70; // 70% of rated power

pub struct ViryaIntegrationContract;

#[contractimpl]
impl ViryaIntegrationContract {
    pub fn initialize(env: Env) {
        env.storage().instance().set(&DEVICES, &Map::new(&env));
        env.storage().instance().set(&MAINTENANCE, &Vec::new(&env));
        env.storage().instance().set(&AUTHORIZED_PROVIDERS, &Vec::<Address>::new(&env));
    }

    pub fn register_inverter(
        env: Env,
        device_id: String,
        policy_id: String,
        manufacturer: String,
        model: String,
        rated_power: u32,
    ) -> Result<(), String> {
        let mut devices: Map<String, Inverter> = env.storage().instance().get(&DEVICES).unwrap_or(Map::new(&env));
        
        if devices.contains_key(&device_id) {
            return Err(String::from_str(&env, "Device already registered"));
        }

        let empty_reading = InverterData {
            timestamp: env.ledger().timestamp(),
            energy_produced: 0,
            peak_power: 0,
            dc_voltage: 0,
            dc_current: 0,
            ac_voltage: 0,
            ac_frequency: 0,
            internal_temp: 0,
            efficiency: 0,
            power_factor: 0,
            daily_yield: 0,
            total_yield: 0,
            operating_hours: 0,
        };

        let device = Inverter {
            device_id: device_id.clone(),
            policy_id,
            last_ping: env.ledger().timestamp(),
            operational_status: true,
            online_status: true,
            risk_level: String::from_str(&env, "LOW"),
            device_type: String::from_str(&env, "INVERTER"),
            manufacturer,
            model,
            rated_power,
            last_reading: empty_reading.clone(),
            hourly_readings: Vec::new(&env),
        };

        devices.set(device_id, device);
        env.storage().instance().set(&DEVICES, &devices);
        Ok(())
    }

    pub fn update_inverter_data(
        env: Env,
        device_id: String,
        auth_provider: Address,
        energy_produced: u64,
        peak_power: u32,
        dc_voltage: u32,
        dc_current: u32,
        ac_voltage: u32,
        ac_frequency: u32,
        internal_temp: i32,
        efficiency: u32,
        power_factor: u32,
        daily_yield: u64,
        total_yield: u64,
        operating_hours: u32,
    ) -> Result<bool, String> {
        // Verify authorized provider
        let authorized: Vec<Address> = env.storage().instance().get(&AUTHORIZED_PROVIDERS).unwrap_or(Vec::new(&env));
        if !authorized.contains(&auth_provider) {
            return Err(String::from_str(&env, "Unauthorized provider"));
        }

        let mut devices: Map<String, Inverter> = env.storage().instance().get(&DEVICES).unwrap_or(Map::new(&env));
        
        if let Some(mut device) = devices.get(device_id.clone()) {
            let new_reading = InverterData {
                timestamp: env.ledger().timestamp(),
                energy_produced,
                peak_power,
                dc_voltage,
                dc_current,
                ac_voltage,
                ac_frequency,
                internal_temp,
                efficiency,
                power_factor,
                daily_yield,
                total_yield,
                operating_hours,
            };

            // Update online status and operational status
            device.online_status = true;
            device.last_ping = env.ledger().timestamp();
            
            // Check if performance is within acceptable range
            let performance_ratio = (peak_power as f64 / device.rated_power as f64 * 100.0) as u32;
            device.operational_status = performance_ratio >= PERFORMANCE_THRESHOLD;
            
            // Update risk level based on operational metrics
            device.risk_level = if device.operational_status && efficiency >= 90 {
                String::from_str(&env, "LOW")
            } else if device.operational_status && efficiency >= 75 {
                String::from_str(&env, "MEDIUM")
            } else {
                String::from_str(&env, "HIGH")
            };

            // Store the reading
            device.last_reading = new_reading.clone();
            device.hourly_readings.push_back(new_reading);
            
            // Keep only last 24 hours of readings
            while device.hourly_readings.len() > 24 {
                device.hourly_readings.remove(0);
            }
            
            devices.set(device_id, device.clone());
            env.storage().instance().set(&DEVICES, &devices);
            
            // Return true if we should trigger a claim process
            return Ok(!device.operational_status && 
                     (env.ledger().timestamp() - device.last_ping) > DOWNTIME_THRESHOLD);
        }
        Err(String::from_str(&env, "Device not found"))
    }

    pub fn check_device_status(
        env: Env,
        device_id: String,
    ) -> Result<(), String> {
        let mut devices: Map<String, Inverter> = env.storage().instance().get(&DEVICES).unwrap_or(Map::new(&env));
        
        if let Some(mut device) = devices.get(device_id.clone()) {
            let current_time = env.ledger().timestamp();
            
            // If no ping received in last 5 minutes, mark device as offline
            if current_time - device.last_ping > 300 {
                device.online_status = false;
                devices.set(device_id, device);
                env.storage().instance().set(&DEVICES, &devices);
            }
            Ok(())
        } else {
            Err(String::from_str(&env, "Device not found"))
        }
    }

    pub fn get_hourly_performance(
        env: Env,
        device_id: String,
    ) -> Result<Vec<InverterData>, String> {
        let devices: Map<String, Inverter> = env.storage().instance().get(&DEVICES).unwrap_or(Map::new(&env));
        
        if let Some(device) = devices.get(device_id) {
            Ok(device.hourly_readings)
        } else {
            Err(String::from_str(&env, "Device not found"))
        }
    }

    // Previous methods remain unchanged
    pub fn add_maintenance_record(
        env: Env,
        device_id: String,
        report_hash: String,
        technician: Address,
        maintenance_type: String,
        issues_found: Vec<String>,
        parts_replaced: Vec<String>,
    ) -> Result<(), String> {
        let devices: Map<String, Inverter> = env.storage().instance().get(&DEVICES).unwrap_or(Map::new(&env));
        
        if !devices.contains_key(&device_id) {
            return Err(String::from_str(&env, "Device not found"));
        }

        let record = MaintenanceRecord {
            timestamp: env.ledger().timestamp(),
            report_hash,
            technician,
            status: String::from_str(&env, "COMPLETED"),
            maintenance_type,
            issues_found,
            parts_replaced,
        };

        let mut maintenance: Vec<MaintenanceRecord> = env.storage().instance().get(&MAINTENANCE).unwrap_or(Vec::new(&env));
        maintenance.push_back(record);
        env.storage().instance().set(&MAINTENANCE, &maintenance);
        Ok(())
    }

    pub fn get_device_status(env: Env, device_id: String) -> Result<Inverter, String> {
        let devices: Map<String, Inverter> = env.storage().instance().get(&DEVICES).unwrap_or(Map::new(&env));
        
        if let Some(device) = devices.get(device_id) {
            Ok(device)
        } else {
            Err(String::from_str(&env, "Device not found"))
        }
    }

    pub fn authorize_provider(env: Env, provider: Address) -> Result<(), String> {
        let mut authorized: Vec<Address> = env.storage().instance().get(&AUTHORIZED_PROVIDERS).unwrap_or(Vec::new(&env));
        authorized.push_back(provider);
        env.storage().instance().set(&AUTHORIZED_PROVIDERS, &authorized);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, Address};

    #[test]
    fn test_inverter_registration_and_data() {
        let env = Env::default();
        let contract_id = env.register_contract(None, ViryaIntegrationContract);
        let client = ViryaIntegrationContractClient::new(&env, &contract_id);

        client.initialize();

        // Register inverter
        let device_id = String::from_str(&env, "INV001");
        let policy_id = String::from_str(&env, "POL001");
        let manufacturer = String::from_str(&env, "SolarEdge");
        let model = String::from_str(&env, "SE7600H");
        let rated_power = 7600;

        let result = client.register_inverter(
            &device_id,
            &policy_id,
            &manufacturer,
            &model,
            &rated_power,
        );
        assert!(result.is_ok());

        // Authorize provider
        let provider = Address::random(&env);
        client.authorize_provider(&provider);

        // Update inverter data
        let result = client.update_inverter_data(
            &device_id,
            &provider,
            5000,    // energy_produced
            6000,    // peak_power
            400,     // dc_voltage
            15000,   // dc_current
            240,     // ac_voltage
            60000,   // ac_frequency
            450,     // internal_temp (45.0°C)
            960,     // efficiency (96.0%)
            980,     // power_factor (98.0%)
            45000,   // daily_yield
            1000000, // total_yield
            12000,   // operating_hours
        );
        assert!(result.is_ok());

        // Check device status
        let device = client.get_device_status(&device_id).unwrap();
        assert_eq!(device.online_status, true);
        assert_eq!(device.operational_status, true);
        assert_eq!(device.last_reading.energy_produced, 5000);
    }
}
