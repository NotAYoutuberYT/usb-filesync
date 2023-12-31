use blake3::Hasher;
use sysinfo::System;
use tracing::trace;

// this whole module could be a lot more simple,
// I'm paranoid my naive method of generating
// ids will change in the future

pub trait IdHashInput {
    fn get_info(&mut self) -> Vec<String>;
}

pub struct SystemAndSalt {
    system: System,
    salt: String,
}

impl SystemAndSalt {
    pub fn new(system: System, salt: String) -> Self {
        Self { system, salt }
    }
}

impl IdHashInput for SystemAndSalt {
    fn get_info(&mut self) -> Vec<String> {
        self.system.refresh_memory();

        let sys_name = System::name().unwrap_or_default();
        let sys_host_name = System::host_name().unwrap_or_default();
        let sys_total_memory = self.system.total_memory().to_string();
        let sys_core_count = self
            .system
            .physical_core_count()
            .unwrap_or_default()
            .to_string();

        trace!(salt = ?self.salt.clone(), ?sys_name, ?sys_host_name, ?sys_total_memory, ?sys_core_count);

        vec![
            self.salt.clone(),
            sys_name,
            sys_host_name,
            sys_total_memory,
            sys_core_count,
        ]
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct SystemId {
    pub id: String,
}

impl SystemId {
    pub fn new_by_hashing(id_info: &mut dyn IdHashInput, hasher: &mut Hasher) -> Self {
        hasher.reset();
        id_info.get_info().iter().for_each(|item| {
            hasher.update(item.as_bytes());
        });

        Self {
            id: hasher.finalize().to_string(),
        }
    }
}

#[cfg(test)]
mod system_id_tests {
    use std::thread;

    use blake3::Hasher;
    use rand::{rngs::StdRng, Rng, SeedableRng};
    use sysinfo::{System, MINIMUM_CPU_UPDATE_INTERVAL};

    use super::{SystemAndSalt, SystemId};

    /// returns two systems, using two different rng seeds for the salt
    fn get_two_system_ids(rng_seed_one: u64, rng_seed_two: u64) -> [SystemId; 2] {
        let mut hasher = Hasher::new();

        let salt = StdRng::seed_from_u64(rng_seed_one).gen::<f32>().to_string();
        let system = System::new();
        let mut system_and_salt = SystemAndSalt::new(system, salt.clone());
        let system_id_one = SystemId::new_by_hashing(&mut system_and_salt, &mut hasher);

        let mut system = System::new();
        system.refresh_all();
        thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);

        let salt = StdRng::seed_from_u64(rng_seed_two).gen::<f32>().to_string();
        let mut system_and_salt = SystemAndSalt::new(system, salt);
        let system_id_two = SystemId::new_by_hashing(&mut system_and_salt, &mut hasher);

        [system_id_one, system_id_two]
    }

    #[test]
    fn id_test_one() {
        let ids = get_two_system_ids(1, 1);
        assert_eq!(ids[0], ids[1])
    }

    #[test]
    fn id_test_two() {
        let ids = get_two_system_ids(2, 2);
        assert_eq!(ids[0], ids[1])
    }

    #[test]
    fn id_test_three() {
        let ids = get_two_system_ids(1, 2);
        assert_ne!(ids[0], ids[1])
    }
}
