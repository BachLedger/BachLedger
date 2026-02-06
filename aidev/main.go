package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"os"
	"strings"
)

const version = "0.1.0"

func main() {
	if len(os.Args) < 2 {
		printUsage()
		os.Exit(1)
	}

	// Handle special commands
	switch os.Args[1] {
	case "-h", "--help", "help":
		printUsage()
		return
	case "-v", "--version", "version":
		fmt.Printf("aidev %s\n", version)
		return
	}

	// Get working directory
	workDir, err := os.Getwd()
	if err != nil {
		fmt.Fprintf(os.Stderr, "❌ Failed to get working directory: %v\n", err)
		os.Exit(1)
	}

	// Generate session ID based on working directory
	sessionID := generateSessionID(workDir)

	// Combine all arguments as the task
	task := strings.Join(os.Args[1:], " ")

	fmt.Printf("🤖 AI Dev Assistant v%s\n", version)
	fmt.Printf("📁 Working directory: %s\n", workDir)
	fmt.Printf("🎯 Task: %s\n", task)
	fmt.Println()

	ctx := context.Background()

	// Phase 1: Planning
	plan, err := runPlanning(ctx, workDir, sessionID, task)
	if err != nil {
		fmt.Fprintf(os.Stderr, "❌ Planning failed: %v\n", err)
		os.Exit(1)
	}

	if len(plan.Tasks) == 0 {
		fmt.Println("ℹ️  No tasks to execute.")
		return
	}

	// Confirm execution
	fmt.Print("\n🚀 Ready to execute. Press Enter to continue, or 'q' to quit: ")
	confirm := askUser("")
	if confirm == "q" || confirm == "quit" {
		fmt.Println("Aborted.")
		return
	}

	// Phase 2: Execution
	if err := runExecution(ctx, workDir, sessionID, plan); err != nil {
		fmt.Fprintf(os.Stderr, "❌ Execution failed: %v\n", err)
		os.Exit(1)
	}

	fmt.Println("\n✅ All tasks completed successfully!")
}

func printUsage() {
	fmt.Printf(`AI Dev Assistant v%s

Usage:
  aidev <task description>

Examples:
  aidev "implement user authentication"
  aidev "fix the bug in login function"
  aidev "add unit tests for utils.go"

Options:
  -h, --help     Show this help message
  -v, --version  Show version

Environment:
  KIMI_API_KEY   API key for Kimi (optional, uses default if not set)
`, version)
}

func generateSessionID(workDir string) string {
	hash := sha256.Sum256([]byte(workDir))
	shortHash := hex.EncodeToString(hash[:4])
	return "aidev-" + shortHash
}
