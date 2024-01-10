use bonsai_bt::Behavior::{Action, RepeatSequence};
use bonsai_bt::{Event, Failure, Running, Status, Success, UpdateArgs, Wait, BT};
use godot::engine::{AnimationPlayer, CharacterBody2D, ICharacterBody2D};
use godot::prelude::*;

#[derive(GodotClass)]
#[class(base = CharacterBody2D)]
pub struct Mob {
    #[base]
    base: Base<CharacterBody2D>,
    pub action_points: usize,
    pub max_action_points: usize,
    pub alive: bool,
    pub bt: BT<EnemyNPC, BlackBoardData>,
    pub animation_completed: bool,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug, PartialEq)]
pub enum EnemyNPC {
    Run,
    Shoot,
    HasActionPointsLeft,
    Rest,
    Die,
    IsDead,
}

#[derive(Debug, Clone)]
pub struct BlackBoardData {
    times_shot: usize,
}

#[godot_api]
impl Mob {
    fn game_tick(&mut self, dt: f64) -> Status {
        let e: Event = UpdateArgs { dt }.into();

        let mut bt = self.bt.clone();
        let status = bt.tick(&e, &mut |args: bonsai_bt::ActionArgs<Event, EnemyNPC>,
                                       blackboard| {
            match *args.action {
                EnemyNPC::Run => {
                    self.perform_action("run");
                    return (Success, dt);
                }
                EnemyNPC::HasActionPointsLeft => {
                    if self.action_points == 0 {
                        println!("NPC does not have actions points left... ");
                        return (Success, dt);
                    } else {
                        println!("NPC has action points: {}", self.action_points);
                        return (Running, dt);
                    }
                }
                EnemyNPC::Shoot => {
                    self.perform_action("shoot");

                    // for the sake of example we get access to blackboard and update
                    // one of its values here:
                    blackboard.get_db().times_shot += 1;

                    return (Success, dt);
                }
                EnemyNPC::Rest => {
                    if self.fully_rested() {
                        return (Success, dt);
                    }
                    self.rest();
                    return (Running, dt);
                }
                EnemyNPC::Die => {
                    self.die();
                    return (Success, dt);
                }
                EnemyNPC::IsDead => {
                    if self.is_alive() {
                        return (Running, dt);
                    }
                    return (Success, dt);
                }
            }
        });
        self.bt = bt;
        return status.0;
    }
    fn consume_action_point(&mut self) {
        self.action_points = self.action_points.saturating_sub(1);
    }
    fn rest(&mut self) {
        if self.animation_completed {
            self.base()
                .get_node_as::<AnimationPlayer>("AnimationPlayer")
                .play_ex()
                .name("sleep".into())
                .done();
            self.animation_completed = false;
            let mut timer = self.base().get_tree().unwrap().create_timer(0.8).unwrap();
            timer.connect(
                "timeout".into(),
                self.base().callable("animation_completed"),
            );
        }
        self.action_points = (self.action_points + 1).min(self.max_action_points);
        println!("{}", self.action_points);
    }
    fn die(&mut self) {
        self.base()
            .get_node_as::<AnimationPlayer>("AnimationPlayer")
            .play_ex()
            .name("die".into())
            .done();
        println!("NPC died...");
        self.alive = false
    }
    fn is_alive(&self) -> bool {
        if self.alive {
            println!("NPC is alive...");
        } else {
            println!("NPC is dead...");
        }
        self.alive
    }
    fn fully_rested(&self) -> bool {
        self.action_points == self.max_action_points && self.animation_completed
    }

    #[func]
    fn animation_completed(&mut self) {
        self.base()
            .get_node_as::<AnimationPlayer>("AnimationPlayer")
            .play_ex()
            .name("RESET".into())
            .done();
        self.animation_completed = true;
    }

    fn perform_action(&mut self, action: &str) {
        if self.action_points > 0 {
            self.consume_action_point();
            println!(
                "Performing action: {}. Action points: {}",
                action, self.action_points
            );
        } else {
            println!(
                "Cannot perform action: {}. Not enough action points.",
                action
            );
        }
    }
}

#[godot_api]
impl ICharacterBody2D for Mob {
    fn init(&base: Base<Self::Base>) -> Self {
        let run_and_shoot_ai = RepeatSequence(
            Box::new(Action(EnemyNPC::HasActionPointsLeft)),
            vec![Action(EnemyNPC::Run), Action(EnemyNPC::Shoot)],
        );
        let top_ai = RepeatSequence(
            Box::new(Action(EnemyNPC::IsDead)),
            vec![
                run_and_shoot_ai.clone(),
                Action(EnemyNPC::Rest),
                Action(EnemyNPC::Die),
            ],
        );

        let blackboard = BlackBoardData { times_shot: 0 };

        let mut bt = BT::new(top_ai, blackboard);
        let print_graph = true;
        if print_graph {
            println!("{}", bt.get_graphviz());
        }
        return Self {
            base,
            bt,
            action_points: 3,
            max_action_points: 3,
            alive: true,
            animation_completed: true,
        };
    }
    fn ready(&mut self) {}

    fn process(&mut self, dt: f64) {
        let status = self.game_tick(dt);
        match status {
            Running => {}
            _ => return,
        }
    }
}
