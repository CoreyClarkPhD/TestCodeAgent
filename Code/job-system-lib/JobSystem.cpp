#include "./JobSystem.h"
#include "./JobWorkerThread.h"
#include "./Job.h"
#include <cstring>
#include <fstream>
#include <iostream>
#include <mutex>
#include <optional>
#include <ostream>
#include <sstream>
#include <string>
#include <thread>
#include <unordered_map>
#include <vector>

JobSystem::JobSystem() {
    this->_workerThreads = std::vector<JobWorkerThread *>();
    this->_jobsQueued = std::deque<std::unique_ptr<Job>>();
    this->_jobsCompleted = std::deque<std::unique_ptr<Job>>();
}

JobSystem *JobSystem::s_jobSystem = nullptr;

JobSystem *JobSystem::CreateOrGet() {
    if (JobSystem::s_jobSystem == nullptr) {
        JobSystem::s_jobSystem = new JobSystem();
    }
    return JobSystem::s_jobSystem;
}

void JobSystem::Destroy() {
    // Shutdown all worker threads
    auto system = JobSystem::CreateOrGet();
    for (auto &workerThread : system->_workerThreads) {
        workerThread->Shutdown();
    }

    // Wait for all threads to be stopped
    for (auto &workerThread : system->_workerThreads) {
        while (!workerThread->IsShutDown()) {
            std::this_thread::yield();
        }
    }

    // Delete all worker threads
    while (system->_workerThreads.size() > 0) {
        auto thread = system->_workerThreads.back();
        system->_workerThreads.pop_back();
        delete thread;
    }

    // Delete all job types
    for (auto &entry : system->jobMap) {
        delete entry.second;
    }
}

void JobSystem::CreateWorkerThread(const std::string id) {
    auto worker = new JobWorkerThread(id);
    this->_workerThreadsMutex.lock();
    this->_workerThreads.emplace_back(worker);
    this->_workerThreadsMutex.unlock();
}

std::string JobSystem::QueueJob(std::string jobType, std::string input) {
    // Get job from map
    auto clonedJob = new Job(jobType, input);
    // Set input
    clonedJob->input = input;
    // Get random id
    std::string id = clonedJob->id;

    // Queue it
    addHistoryEntry(JobHistoryEntry(jobType, JobStatus::QUEUED));
    this->_jobsQueuedMutex.lock();
    this->_jobsQueued.emplace_back(std::unique_ptr<Job>(clonedJob));
    this->_jobsQueuedMutex.unlock();

    return id;
}

void JobSystem::setResultFromWorker(std::string id, std::string result) {
    this->resultsMutex.lock();
    this->results[id] = result;
    this->resultsMutex.unlock();
}

std::string JobSystem::runJob(std::string jobType, std::string jobInput) {
    std::string id = this->QueueJob(jobType, jobInput);
    // Wait for job to complete
    while (!this->IsJobComplete(id)) {
        std::this_thread::yield();
    }
    // Get job result
    this->resultsMutex.lock();
    std::string result = this->results[id];
    this->resultsMutex.unlock();
    // Copy result and return
    return result;
}

std::optional<std::unique_ptr<Job>> JobSystem::ClaimAJob() {
    this->_jobsQueuedMutex.lock();
    if (this->_jobsQueued.size() > 0) {
        auto job = std::move(this->_jobsQueued.front());
        addHistoryEntry(JobHistoryEntry(job.get()->id, JobStatus::RUNNING));
        this->_jobsQueued.pop_front();
        this->_jobsQueuedMutex.unlock();
        return job;
    }
    this->_jobsQueuedMutex.unlock();
    return std::nullopt;
}

bool JobSystem::IsJobComplete(std::string jobId) const {
    this->_jobsCompletedMutex.lock();
    for (auto &job : this->_jobsCompleted) {
        if (job.get()->id == jobId) {
            this->_jobsCompletedMutex.unlock();
            return true;
        }
    }
    this->_jobsCompletedMutex.unlock();
    return false;
}

bool JobSystem::HasJobsActive() {
    std::lock_guard<std::mutex> guard(this->_jobsQueuedMutex);
    std::lock_guard<std::mutex> guard2(this->_workerThreadsMutex);

    // If queue has anything in it return false
    if (this->_jobsQueued.size() > 0) {
        return true;
    }

    // Check all worker threads
    for (auto &workerThread : this->_workerThreads) {
        if (workerThread->IsBusy()) {
            return true;
        }
    }

    return false;
}

void JobSystem::MarkJobComplete(std::unique_ptr<Job> job) {
    addHistoryEntry(JobHistoryEntry(job.get()->id, JobStatus::COMPLETED));
    this->_jobsCompletedMutex.lock();
    this->_jobsCompleted.emplace_back(std::move(job));
    this->_jobsCompletedMutex.unlock();
}

void JobSystem::addHistoryEntry(JobHistoryEntry entry) {
    this->m_jobHistoryMutex.lock();
    this->m_jobHistory.emplace_back(entry);
    this->m_jobHistoryMutex.unlock();
}

void JobSystem::DumpHistoryToFile(const std::string &filename) const {
    std::ofstream file;
    file.open(filename);

    for (auto &entry : this->m_jobHistory) {
        file << "JobId: " << (entry.jobId)
             << " JobStatus: " << static_cast<int>(entry.m_jobStatus)
             << std::endl;
    }

    file.close();
}

std::vector<JobHistoryEntry> JobSystem::GetCurrentHistory() const {
    return this->m_jobHistory;
}

void JobSystem::registerJobType(std::string name, Job *job) {
    this->jobMap[name] = job;
}

std::vector<std::string> JobSystem::listJobTypes() {
    std::vector<std::string> result;
    for (auto &entry : this->jobMap) {
        result.emplace_back(entry.first);
    }
    return result;
}

JobStatus JobSystem::GetJobStatus(std::string jobId) const {
    this->m_jobHistoryMutex.lock();
    for (auto &entry : this->m_jobHistory) {
        if (entry.jobId == jobId) {
            this->m_jobHistoryMutex.unlock();
            return entry.m_jobStatus;
        }
    }
    this->m_jobHistoryMutex.unlock();
    return JobStatus::NEVER_SEEN;
}

void JobSystem::DestroyJob(std::string jobId) {
    // Search the jobsqued mutex
    this->_jobsQueuedMutex.lock();
    for (auto it = this->_jobsQueued.begin(); it != this->_jobsQueued.end();
         ++it) {
        if (it->get()->id == jobId) {
            this->_jobsQueued.erase(it);
            this->_jobsQueuedMutex.unlock();
            return;
        }
    }
    this->_jobsQueuedMutex.unlock();

    // Search the jobscompleted mutex
    this->_jobsCompletedMutex.lock();
    for (auto it = this->_jobsCompleted.begin();
         it != this->_jobsCompleted.end(); ++it) {
        if (it->get()->id == jobId) {
            this->_jobsCompleted.erase(it);
            this->_jobsCompletedMutex.unlock();
            return;
        }
    }
    this->_jobsCompletedMutex.unlock();
}
