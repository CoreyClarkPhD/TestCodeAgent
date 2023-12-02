#include "./JobWorkerThread.h"
#include "./JobSystem.h"
#include "./Types.h"
#include <iostream>
#include <mutex>
#include <ostream>
#include <thread>

extern "C" {
const char *run_rust_job(const char *job_type, const char *job_input);
}

JobWorkerThread::JobWorkerThread(std::string id) {
    this->id = id;
    this->isStopping = false;
    m_thread = new std::thread(&JobWorkerThread::Work, this);
}

bool JobWorkerThread::IsBusy() const { return hasJobToDo; }

void JobWorkerThread::Work() {
    // Get a job from the job system
    hasJobToDo = true;
    auto system = JobSystem::CreateOrGet();

    while (true) {
        if (this->isStopping) {
            std::cout << "JobWorkerThread::Work() is stopping" << std::endl;
            return;
        }
        if (this->_job.get() == nullptr) {
            hasJobToDo = true;
            auto possiblejob = JobSystem::CreateOrGet()->ClaimAJob();
            if (possiblejob.has_value()) {
                this->_job = std::move(possiblejob.value());
            } else {
                std::this_thread::yield();
                hasJobToDo = false;
            }
        }
        if (this->_job.get() != nullptr) {
            std::cout << "JobWorkerThread::Work() has a job" << this->_job->type << std::endl;
            // THIS IS WHERE THE NEW MAGIC HAPPENS
            const char* result = run_rust_job(this->_job->type.c_str(), this->_job->input.c_str());

            std::cout << "JobWorkerThread::Work() got result " << result << std::endl;
            // this->_job->Execute(_job->input.value());

            // Move job to complete queue
            system->setResultFromWorker(this->_job->id, result);
            system->MarkJobComplete(std::move(this->_job));
            this->_job = std::unique_ptr<Job>();
            hasJobToDo = true;
        }
    }
}

void JobWorkerThread::Shutdown() {
    std::cout << "JobWorkerThread::Shutdown()" << std::endl;
    this->isStopping = true;
}

bool JobWorkerThread::IsShutDown() const {
    return this->_job.get() == nullptr && this->isStopping;
}
