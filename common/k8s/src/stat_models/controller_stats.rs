pub struct ControllerStats {
    containers_ready: i32,
    containers_total: i32,
    pods_ready: i32,
    pods_total: i32
}

impl ControllerStats {

    pub fn build() -> ControllerStats {

        ControllerStats {
            containers_ready: 0,
            containers_total: 0,
            pods_ready: 0,
            pods_total: 0,
        }  
    }

    pub fn inc_containers_ready(&mut self) {
        self.containers_ready += 1;
    }

    pub fn inc_containers_total(&mut self) {
        self.containers_total += 1;
    }

    pub fn inc_pods_ready(&mut self) {
        self.pods_ready += 1;
    }

    pub fn inc_pods_total(&mut self) {
        self.pods_total += 1;
    }

}