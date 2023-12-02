#pragma once
#include "./Job.h"
#include "JobWorkerThread.h"
#include "Types.h"
#include <atomic>
#include <deque>
#include <memory>
#include <mutex>
#include <optional>
#include <string>
#include <unordered_map>
#include <vector>

using JobMap = std::unordered_map<std::string, Job *>;

class JobSystem {
  public:
    JobSystem();
    ~JobSystem(){};
    static JobSystem *CreateOrGet();

    static void Destroy();

    void CreateWorkerThread(const std::string id);

    std::string QueueJob(std::string jobType, std::string input);

    void MarkJobComplete(std::unique_ptr<Job> job);

    std::optional<std::unique_ptr<Job>> ClaimAJob();

    bool IsJobComplete(std::string jobId) const;

    JobStatus GetJobStatus(std::string jobId) const;

    bool HasJobsActive();

    void DumpHistoryToFile(const std::string &filename) const;
    std::vector<JobHistoryEntry> GetCurrentHistory() const;

    std::string CompleteJob(std::string jobId);

    void DestroyJob(std::string jobId);

    std::vector<std::string> listJobTypes();

    std::string runJob(std::string jobType, std::string input);

    void setResultFromWorker(std::string id, std::string result);

    static JobSystem *s_jobSystem;
    JobMap jobMap;

  private:
    void addHistoryEntry(JobHistoryEntry entry);
    void registerJobType(std::string name, Job *job);

    std::vector<JobWorkerThread *> _workerThreads;
    mutable std::mutex _workerThreadsMutex;

    std::deque<std::unique_ptr<Job>> _jobsQueued;
    std::deque<std::unique_ptr<Job>> _jobsCompleted;

    mutable std::mutex _jobsQueuedMutex;
    mutable std::mutex _jobsCompletedMutex;

    std::vector<JobHistoryEntry> m_jobHistory;
    mutable std::mutex m_jobHistoryMutex;

    std::unordered_map<std::string, std::string> results;
    mutable std::mutex resultsMutex;
};
