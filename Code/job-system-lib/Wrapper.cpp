#include "JobSystem.h"
#include "RandomId.h"
#include <iostream>
#include <random>
#include <string.h>

extern "C" {

void CreateWorkerThread() {
    JobSystem *system = JobSystem::CreateOrGet();
    system->CreateWorkerThread(generateRandomID(10));
}

char *QueueJob(char *jobType, char *input) {
    JobSystem *system = JobSystem::CreateOrGet();
    auto id = system->QueueJob(jobType, input);
    return strdup(id.c_str());
}

bool HasJobsActive() {
    JobSystem *system = JobSystem::CreateOrGet();
    return system->HasJobsActive();
}

void Destroy() {
    JobSystem *system = JobSystem::CreateOrGet();
    delete system;
}

bool IsJobComplete(char *jobId) {
    JobSystem *system = JobSystem::CreateOrGet();
    return system->IsJobComplete(jobId);
}

int GetJobStatus(char *jobId) {
    try {
        JobSystem *system = JobSystem::CreateOrGet();
        return (int)system->GetJobStatus(jobId);

    } catch (const std::exception &) {
        return -1;
    }
    return -1;
}

void DumpHistoryToFile(char *filename) {
    JobSystem *system = JobSystem::CreateOrGet();
    system->DumpHistoryToFile(filename);
}

void DestroyJob(char *jobId) {
    JobSystem *system = JobSystem::CreateOrGet();
    system->DestroyJob(jobId);
}

char *RunJob(char *jobType, char *jobInput) {
    JobSystem *system = JobSystem::CreateOrGet();
    auto result = system->runJob(jobType, jobInput);
    return strdup(result.c_str());
}

}
