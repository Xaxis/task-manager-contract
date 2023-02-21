// Import the necessary NEAR SDK items
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{UnorderedMap, Vector},
    env,
    near_bindgen,
};

// Define the data structure for a task
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Task {
    image_url: String,
    description: String,
    assigned_to: Option<String>,
    assigned_at: Option<u64>,
    reviewed_by: Option<String>,
    reviewed_at: Option<u64>,
}

// Define the data structure for a review task
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ReviewTask {
    task_id: u64,
    reviewed_by: String,
    reviewed_at: u64,
    is_accepted: bool,
}

// Define the smart contract struct
#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct TaskManager {
    tasks: UnorderedMap<u64, Task>,
    task_queue: Vector<u64>,
    review_tasks: UnorderedMap<u64, ReviewTask>,
    review_queue: Vector<u64>,
}

impl Default for TaskManager {
    fn default() -> Self {
        Self {
            tasks: UnorderedMap::new(b"tasks".to_vec()),
            task_queue: Vector::new(b"task_queue".to_vec()),
            review_tasks: UnorderedMap::new(b"review_tasks".to_vec()),
            review_queue: Vector::new(b"review_queue".to_vec()),
        }
    }
}

// Implement the public methods for the smart contract
#[near_bindgen]
impl TaskManager {
      
    // Method to add a new task to the queue
    pub fn add_task(&mut self, image_url: String) {
        let task_id = self.tasks.len() as u64;
        let task = Task {
            image_url,
            description: "".to_string(),
            assigned_to: None,
            assigned_at: None,
            reviewed_by: None,
            reviewed_at: None,
        };
        self.tasks.insert(&task_id, &task);
        self.task_queue.push(&task_id);
    }

    // Method to assign a task to a user
    pub fn assign_task(&mut self, task_id: u64, user_account: String) {
        assert!(self.task_queue.contains(&task_id), "Task not in queue");
        let mut task = self.tasks.get(&task_id).unwrap();
        task.assigned_to = Some(user_account.clone());
        task.assigned_at = Some(env::block_timestamp());
        self.tasks.insert(&task_id, &task);
        self.task_queue.remove(self.task_queue.iter().position(|x| *x == task_id).unwrap());
    }

    // Method to submit a task for review
    pub fn submit_task(&mut self, task_id: u64, description: String) {
        let mut task = self.tasks.get(&task_id).unwrap();
        assert_eq!(task.assigned_to, Some(env::predecessor_account_id()), "You are not assigned to this task");
        task.description = description;
        self.tasks.insert(&task_id, &task);
        let review_task_id = self.review_tasks.len() as u64;
        let review_task = ReviewTask {
            task_id,
            reviewed_by: "".to_string(),
            reviewed_at: 0,
            is_accepted: false,
        };
        self.review_tasks.insert(&review_task_id, &review_task);
        self.review_queue.push(&review_task_id);
    }
      
    // Method to assign a review task to a user
    pub fn assign_review_task(&mut self, review_task_id: u64, user_account: String) {
        assert!(self.review_queue.contains(&review_task_id), "Review task not in queue");
        let mut review_task = self.review_tasks.get(&review_task_id).unwrap();
        review_task.reviewed_by = user_account.clone();
        review_task.reviewed_at = env::block_timestamp();
        self.review_tasks.insert(&review_task_id, &review_task);
        self.review_queue.remove(self.review_queue.iter().position(|x| *x == review_task_id).unwrap());
    }
      
    // Method to accept or reject a task review
    pub fn review_task(&mut self, review_task_id: u64, is_accepted: bool) {
        assert_eq!(self.review_tasks.get(&review_task_id).unwrap().reviewed_by, Some(env::predecessor_account_id()), "You are not assigned to this review task");
        let mut review_task = self.review_tasks.get(&review_task_id).unwrap();
        review_task.is_accepted = is_accepted;
        self.review_tasks.insert(&review_task_id, &review_task);
        let task = self.tasks.get(&review_task.task_id).unwrap();
        if is_accepted {
            self.tasks.remove(&review_task.task_id);
            env::log(format!("Task {} accepted by reviewer {}", review_task.task_id, env::predecessor_account_id()).as_bytes());
        } else {
            self.task_queue.push(&review_task.task_id);
            env::log(format!("Task {} rejected by reviewer {}", review_task.task_id, env::predecessor_account_id()).as_bytes());
        }
        self.review_tasks.remove(&review_task_id);
    }
      
    // Method to get the task queue
    pub fn get_task_queue(&self) -> Vec<u64> {
        self.task_queue.to_vec()
    }

    // Method to get the review queue
    pub fn get_review_queue(&self) -> Vec<u64> {
        self.review_queue.to_vec()
    }
}
