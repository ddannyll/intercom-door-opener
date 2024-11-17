use std::collections::HashSet;

use tokio::sync::broadcast;

#[derive(Debug, Clone, PartialEq)]
pub enum IntercomState {
    Setup,
    Waiting,
    Moving,
    SettingInactive,
    SettingActive,
}

#[derive(Debug)]
pub enum IntercomStateChangeEvent {
    SetupDone { task: IntercomSetupTasks },
    SignalServoToOpen,
    ServoDoneOpening,
    ButtonPress,
}

#[derive(Hash, PartialEq, Eq, Debug)]
pub enum IntercomSetupTasks {
    SetupServo = 0,
    SetupWifi = 1,
}

pub type Angle = u8;

pub struct Intercom {
    state: IntercomState,
    servo_inactive_angle: Angle,
    servo_active_angle: Angle,
    setup_tasks_remaining: HashSet<IntercomSetupTasks>,
    change_tx: broadcast::Sender<IntercomState>,
}

impl Intercom {
    pub fn new(change_tx: broadcast::Sender<IntercomState>) -> Self {
        Intercom {
            state: IntercomState::Setup,
            servo_active_angle: 30, // TODO: save/load active angles from storage
            servo_inactive_angle: 0,
            change_tx,
            setup_tasks_remaining: HashSet::from_iter(vec![
                IntercomSetupTasks::SetupWifi,
                IntercomSetupTasks::SetupServo,
            ]),
        }
    }
    pub fn change_state(mut self, event: IntercomStateChangeEvent) {
        let prev_state = self.state.clone();
        match (&mut self.state, &event) {
            // Setup Tasks
            (IntercomState::Setup, IntercomStateChangeEvent::SetupDone { task }) => {
                if !self.setup_tasks_remaining.remove(&task) {
                    log::warn!(
                        "Attempted to setup task {:?} however it was already setup",
                        &task
                    );
                }
                if self.setup_tasks_remaining.is_empty() {
                    self.state = IntercomState::Waiting;
                }
            }

            // Button Click
            (IntercomState::Waiting, IntercomStateChangeEvent::ButtonPress) => {
                self.state = IntercomState::SettingInactive
            }
            (IntercomState::SettingInactive, IntercomStateChangeEvent::ButtonPress) => {
                self.state = IntercomState::SettingActive
            }
            (IntercomState::SettingActive, IntercomStateChangeEvent::ButtonPress) => {
                self.state = IntercomState::Waiting
            }

            // Signal opening
            (IntercomState::Waiting, IntercomStateChangeEvent::SignalServoToOpen) => {
                self.state = IntercomState::Moving
            }
            (IntercomState::Moving, IntercomStateChangeEvent::ServoDoneOpening) => {
                self.state = IntercomState::Waiting
            }
            _ => {
                log::warn!(
                    "Unimplemented state change:\n{:?} -- {:?} --> ?",
                    &self.state,
                    &event
                );
            }
        }
        if self.state != prev_state {
            log::info!(
                "State change:\n{:?} -- {:?} --> {:?}",
                prev_state,
                &self.state,
                &event
            );
            let _ = self.change_tx.send(self.state);
        }
    }

    pub fn get_state(self) -> IntercomState {
        self.state
    }

    pub fn get_servo_active_angle(self) -> Angle {
        self.servo_active_angle
    }

    pub fn get_servo_inactive_angle(self) -> Angle {
        self.servo_inactive_angle
    }
}
