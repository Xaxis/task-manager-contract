// Import the necessary NEAR SDK items
use near_sdk::{
    borsh::{self, BorshDeserialize, BorshSerialize},
    collections::{UnorderedMap, Vector},
    env,
    near_bindgen,
    serde::{Deserialize, Serialize},
    json_types::{ValidAccountId, U128},
    Balance, PanicOnDefault, Promise,
};

// Define the data structure for a task
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct Task {
    image_url: String,
    descriptions: String,
    assigned_to: Option<String>,
    assigned_at: Option<u64>,
    reviewed_by: Option<String>,
    reviewed_at: Option<u64>,
    is_completed: bool,
}

// Define the data structure for a review task
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
pub struct ReviewTask {
    task_id: u64,
    reviewed_by: String,
    reviewed_at: u64,
    is_accepted: bool,
}

// Define the main contract structure
#[near_bindgen]
#[derive(PanicOnDefault, BorshDeserialize, BorshSerialize)]
pub struct TaskManager {
    tasks: UnorderedMap<u64, Task>,
    review_tasks: UnorderedMap<u64, ReviewTask>,
    task_queue: Vector<u64>,
    review_queue: Vector<u64>,
    task_counter: u64,
    review_counter: u64,
    payout_account: String,
}

// Implement the public methods for the smart contract
#[near_bindgen]
impl TaskManager {
      
    // Method to add a new task to the queue
    pub fn add_task(&mut self, image_url: String) {
      let task_id = self.task_counter;
        let task = Task {
            image_url,
            descriptions,
            assigned_to: None,
            assigned_at: None,
            reviewed_by: None,
            reviewed_at: None,
            is_completed: false,
        };
        self.tasks.insert(&task_id, &task);
        self.task_queue.push(&task_id);
        self.task_counter += 1;
        task_id
    }

    // Method to assign a task to a user
    pub fn assign_task(&mut self, task_id: u64, user_account: String) {
        assert!(self.task_queue.contains(&task_id), "Task not in queue");
        let mut task = self.tasks.get(&task_id).unwrap();
        assert!(task.assigned_to.is_none(), "Task already assigned");
        task.assigned_to = Some(user_account.clone());
        task.assigned_at = Some(env::block_timestamp());
        self.tasks.insert(&task_id, &task);
        self.task_queue.remove(self.task_queue.iter().position(|x| *x == task_id).unwrap());
        self.review_queue.push(&self.review_counter);
        let review_task = ReviewTask {
            task_id,
            reviewed_by: None,
            reviewed_at: None,
            is_accepted: false,
        };
        self.review_tasks.insert(&self.review_counter, &review_task);
        self.review_counter += 1;
    }

    // Method to submit a task for review
    pub fn submit_task(&mut self, task_id: u64, descriptions: [Option<String>; 4]) {
        let task = self.tasks.get(&task_id).expect("Invalid task ID");
        assert_eq!(task.assigned_to, Some(env::predecessor_account_id()), "You are not assigned to this task");
        assert_eq!(task.is_completed, false, "Task is already completed");
        let new_descriptions = descriptions
            .iter()
            .filter_map(|desc| desc.clone())
            .collect::<Vec<String>>()
            .join(";");
        let updated_task = Task {
            image_url: task.image_url.clone(),
            descriptions: new_descriptions,
            assigned_to: task.assigned_to.clone(),
            assigned_at: task.assigned_at,
            reviewed_by: task.reviewed_by.clone(),
            reviewed_at: task.reviewed_at,
            is_completed: true,
        };
        self.tasks.insert(&task_id, &updated_task);
        let review_task_id = self.review_tasks.len() as u64;
        let review_task = ReviewTask {
            task_id,
            reviewed_by: None,
            reviewed_at: None,
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
        assert!(self.review_queue.contains(&review_task_id), "Review task not in queue");
        let mut review_task = self.review_tasks.get(&review_task_id).unwrap();
        assert!(review_task.reviewed_by.is_none(), "Review task already assigned");
        review_task.reviewed_by = Some(user_account.clone());
        review_task.reviewed_at = Some(env::block_timestamp());
        self.review_tasks.insert(&review_task_id, &review_task);
        self.review_queue.remove(self.review_queue.iter().position(|x| *x == review_task_id).unwrap());
        let task = self.tasks.get(&review_task.task_id).unwrap();
        if task.descriptions.iter().all(|d| d.is_some()) {
            review_task.is_accepted = true;
            self.review_tasks.insert(&review_task_id, &review_task);
            let payout_amount = U128(1_000_000_000_000_000_000_000_000); // 1 NEAR in yoctoNEAR
            let _promise = Promise::new(self.payout_account.clone())
                .transfer(payout_amount.0);
        } else {
            self.review_tasks.insert(&review_task_id, &review_task);
            self.review_queue.push(&review_task_id);
        }
    }
      
    // Method to get the review queue
    pub fn get_review_queue(&self) -> Vec<u64> {
        self.review_queue.to_vec()
    }

    // Method to get the details of a task
    pub fn get_task(&self, task_id: u64) -> Option<Task> {
        self.tasks.get(&task_id)
    }

    // Method to get the details of a review task
    pub fn get_review_task(&self, review_task_id: u64) -> Option<ReviewTask> {
        self.review_tasks.get(&review_task_id)
    }

    // Method to get the number of tasks in the queue
    pub fn get_task_queue_len(&self) -> u64 {
        self.task_queue.len()
    }

    // Method to get the number of tasks in the review queue
    pub fn get_review_queue_len(&self) -> u64 {
        self.review_queue.len()
    }
}
