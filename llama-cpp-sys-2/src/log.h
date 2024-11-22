// referenced by llava.cpp
#pragma once

#ifndef PATCH_LOG_H
#define PATCH_LOG_H
#include <stdio.h>

#define LOG_TEE printf

#define die(msg)          do { fputs("error: " msg "\n", stderr);                exit(1); } while (0)
#define die_fmt(fmt, ...) do { fprintf(stderr, "error: " fmt "\n", __VA_ARGS__); exit(1); } while (0)

#endif // PATCH_LOG_H