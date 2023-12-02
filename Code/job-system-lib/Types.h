#pragma once

#include <optional>
#include <string>
#include <unordered_map>

enum class JobStatus {
    NEVER_SEEN,
    QUEUED,
    RUNNING,
    COMPLETED,
    NUM_JOB_STATUSES
};


struct JobHistoryEntry {
    JobHistoryEntry(std::string jobId, JobStatus jobStatus)
        : jobId(jobId), m_jobStatus(jobStatus) {}
    std::string jobId;
    JobStatus m_jobStatus = JobStatus::NEVER_SEEN;
};
