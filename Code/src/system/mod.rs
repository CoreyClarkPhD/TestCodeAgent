use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::system::types::JobType;

pub mod job_core;
pub mod types;
extern "C" {
    fn Log(message: *const libc::c_char, level: i32);
    fn HasJobsActive() -> bool;
    fn Destroy();
    fn IsJobComplete(jobId: *const libc::c_char) -> bool;
    fn GetJobStatus(jobId: *const libc::c_char) -> i32;
    fn DumpHistoryToFile(filename: *const libc::c_char);
    fn DestroyJob(jobId: *const libc::c_char);
    fn RunJob(jobType: *const libc::c_char, input: *const libc::c_char) -> *const libc::c_char;
    fn CreateWorkerThread();
}

pub enum JobStatus {
    NeverSeen,
    Queued,
    Running,
    Completed,
    Other,
}

pub fn log_level(message: &str, level: i32) {
    let c_message = std::ffi::CString::new(message).unwrap();
    unsafe {
        Log(c_message.as_ptr(), level);
    }
}

pub fn log(message: &str) {
    let c_message = std::ffi::CString::new(message).unwrap();
    unsafe {
        Log(c_message.as_ptr(), 3);
    }
}

pub fn has_jobs_active() -> bool {
    unsafe { HasJobsActive() }
}

pub fn destroy() {
    unsafe { Destroy() }
}

pub fn is_job_complete(job_id: &str) -> bool {
    let c_job_id = std::ffi::CString::new(job_id).unwrap();
    unsafe { IsJobComplete(c_job_id.as_ptr()) }
}

pub fn get_job_status(job_id: &str) -> Option<JobStatus> {
    let c_job_id = std::ffi::CString::new(job_id).unwrap();
    let status = unsafe { GetJobStatus(c_job_id.as_ptr()) };

    if status == -1 {
        return None;
    }

    let status = match status {
        0 => JobStatus::NeverSeen,
        1 => JobStatus::Queued,
        2 => JobStatus::Running,
        3 => JobStatus::Completed,
        _ => JobStatus::Other,
    };

    Some(status)
}

pub fn dump_history_to_file(filename: &str) {
    let c_filename = std::ffi::CString::new(filename).unwrap();
    unsafe { DumpHistoryToFile(c_filename.as_ptr()) }
}

pub fn destroy_job(job_id: &str) {
    let c_job_id = std::ffi::CString::new(job_id).unwrap();
    unsafe { DestroyJob(c_job_id.as_ptr()) }
}

// Run a job in the normal code
pub fn run_job<'a, T: job_core::Job + Serialize + Deserialize<'a>>(
    job_type: JobType,
    input: T,
) -> Value {
    let c_job_type = std::ffi::CString::new(serde_json::to_string(&job_type).unwrap()).unwrap();
    let input_json = serde_json::to_string(&input).unwrap();
    let c_input = std::ffi::CString::new(input_json).unwrap();
    let c_result = unsafe { RunJob(c_job_type.as_ptr(), c_input.as_ptr()) };
    let result = unsafe {
        std::ffi::CStr::from_ptr(c_result)
            .to_str()
            .unwrap()
            .to_string()
            .clone()
    };

    serde_json::from_str(result.as_str()).expect("Valid json")
}

// Used for running a job in flowscript
pub fn run_job_fs(job_type: String, input: Value) -> Value {
    let c_job_type = std::ffi::CString::new(serde_json::to_string(&job_type).unwrap()).unwrap();
    let input_json = serde_json::to_string(&input).unwrap();
    let c_input = std::ffi::CString::new(input_json).unwrap();
    let c_result = unsafe { RunJob(c_job_type.as_ptr(), c_input.as_ptr()) };
    let result = unsafe {
        std::ffi::CStr::from_ptr(c_result)
            .to_str()
            .unwrap()
            .to_string()
            .clone()
    };

    serde_json::from_str(result.as_str()).expect("Valid json")
}

pub fn create_worker_thread() {
    unsafe { CreateWorkerThread() }
}

#[no_mangle]
pub extern "C" fn run_rust_job(
    job_type: *const libc::c_char,
    job_input: *const libc::c_char,
) -> *const libc::c_char {
    unsafe {
        let job_type = std::ffi::CStr::from_ptr(job_type)
            .to_str()
            .unwrap()
            .to_string()
            .clone();
        let input = std::ffi::CStr::from_ptr(job_input)
            .to_str()
            .unwrap()
            .to_string()
            .clone();
        let input: serde_json::Value = serde_json::from_str(input.as_str()).expect("Valid json");

        let jobtype: JobType = serde_json::from_str(&job_type).unwrap();
        println!("C++ : {}, enum : {:?}", job_type, jobtype);
        let result = crate::system::job_core::run_job(jobtype, input);

        let c_result = Box::into_raw(Box::new(
            std::ffi::CString::new(result.to_string()).unwrap(),
        ));

        return c_result.as_ref().unwrap().as_ptr();
    }
}
