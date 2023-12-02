#pragma once
#include "Job.h"
#include "Types.h"
#include <atomic>
#include <memory>
#include <mutex>
#include <optional>
#include <string>
#include <thread>

class JobWorkerThread {

  public:
    JobWorkerThread(std::string id);
    ~JobWorkerThread() {
        if (m_thread != nullptr) {
            m_thread->join();
            delete m_thread;
        }
    }

    void Shutdown(); // Signal that work should stop at next opportunity, only
                     // sends signal

    bool IsShutDown() const;
    bool IsBusy() const;

  private:
    void Work(); // Called in private thread, blocks forever until StopWorking()
    mutable std::mutex _jobMutex;
    std::unique_ptr<Job> _job = std::unique_ptr<Job>();
    std::string id;
    std::atomic<bool> isStopping;
    std::atomic<bool> hasJobToDo = true;
    std::thread *m_thread = nullptr;
};
