package main

import (
	"context"
	"fmt"
	"strings"
)

const maxRetries = 3

// runPlanning executes the planning phase
func runPlanning(ctx context.Context, workDir, sessionID, task string) (*PlannerOutput, error) {
	fmt.Println("📋 Analyzing task...")

	planner, err := NewAgent("planner", workDir, sessionID)
	if err != nil {
		return nil, fmt.Errorf("failed to create planner: %w", err)
	}
	defer planner.Close()

	// First pass: analyze and generate questions
	response, err := planner.Run(ctx, "Task: "+task)
	if err != nil {
		return nil, fmt.Errorf("planner failed: %w", err)
	}

	plan, err := parsePlannerOutput(response)
	if err != nil {
		// If parsing fails, show raw response
		fmt.Println("\n⚠️  Could not parse structured output. Raw response:")
		fmt.Println(response)
		return nil, err
	}

	// Show analysis
	if plan.Analysis != "" {
		fmt.Println("\n📊 Analysis:")
		fmt.Println(plan.Analysis)
	}

	// If there are questions, ask user
	if len(plan.Questions) > 0 || len(plan.ResourcesNeeded) > 0 {
		printQuestions(plan.Questions)
		printResources(plan.ResourcesNeeded)

		// Get user answers
		fmt.Println("Press Enter to accept suggestions, or type your answers:")
		answers := askUser("> ")

		if answers != "" {
			// Re-run planner with user answers
			followUp := fmt.Sprintf("Task: %s\n\nUser's answers to questions:\n%s", task, answers)
			response, err = planner.Run(ctx, followUp)
			if err != nil {
				return nil, fmt.Errorf("planner follow-up failed: %w", err)
			}
			plan, err = parsePlannerOutput(response)
			if err != nil {
				return nil, err
			}
		}
	}

	printTasks(plan.Tasks)
	return plan, nil
}

// runExecution executes all tasks from the plan
func runExecution(ctx context.Context, workDir, sessionID string, plan *PlannerOutput) error {
	fmt.Println("\n🚀 Starting execution...")

	for _, task := range plan.Tasks {
		fmt.Printf("\n▶️  Executing task [%s]: %s\n", task.ID, task.Description)

		if err := executeTask(ctx, workDir, sessionID, task); err != nil {
			return fmt.Errorf("task %s failed: %w", task.ID, err)
		}

		fmt.Printf("✅ Task [%s] completed\n", task.ID)
	}

	return nil
}

// executeTask executes a single task with retry logic
func executeTask(ctx context.Context, workDir, sessionID string, task Task) error {
	taskPrompt := buildTaskPrompt(task)

	for retry := 0; retry < maxRetries; retry++ {
		if retry > 0 {
			fmt.Printf("   🔄 Retry %d/%d\n", retry, maxRetries-1)
		}

		// Run coder
		coder, err := NewAgent("coder", workDir, sessionID+"-coder")
		if err != nil {
			return err
		}

		response, err := coder.Run(ctx, taskPrompt)
		coder.Close()
		if err != nil {
			return err
		}

		coderOutput, err := parseCoderOutput(response)
		if err != nil {
			fmt.Println("   ⚠️  Could not parse coder output")
			continue
		}

		// Check if coder needs clarification
		if coderOutput.Status == "needs_clarification" {
			fmt.Println("   ❓ Coder needs clarification:")
			fmt.Println("   " + coderOutput.Notes)
			answer := askUser("   > ")
			taskPrompt = taskPrompt + "\n\nUser clarification: " + answer
			continue
		}

		// Check if coder is blocked
		if coderOutput.Status == "blocked" {
			fmt.Println("   ⚠️  Coder is blocked: " + coderOutput.Notes)
			return fmt.Errorf("coder blocked: %s", coderOutput.Notes)
		}

		// Report issues found
		if len(coderOutput.IssuesFound) > 0 {
			fmt.Println("   ⚠️  Issues found during implementation:")
			for _, issue := range coderOutput.IssuesFound {
				fmt.Printf("      • [%s] %s\n", issue.Type, issue.Description)
			}
		}

		// Run critic
		critic, err := NewAgent("critic", workDir, sessionID+"-critic")
		if err != nil {
			return err
		}

		criticPrompt := buildCriticPrompt(task, coderOutput)
		response, err = critic.Run(ctx, criticPrompt)
		critic.Close()
		if err != nil {
			return err
		}

		criticOutput, err := parseCriticOutput(response)
		if err != nil {
			fmt.Println("   ⚠️  Could not parse critic output")
			continue
		}

		// Check critic verdict
		if criticOutput.Verdict == "approved" {
			fmt.Println("   ✅ Code review passed")
			return nil
		}

		// Show critic feedback
		fmt.Println("   ❌ Code review failed:")
		for _, issue := range criticOutput.Issues {
			fmt.Printf("      • [%s] %s: %s\n", issue.Severity, issue.Category, issue.Description)
		}

		// Add feedback to task prompt for next retry
		taskPrompt = taskPrompt + "\n\nPrevious review feedback:\n" + criticOutput.Summary
	}

	return fmt.Errorf("task failed after %d retries", maxRetries)
}

// buildTaskPrompt builds the prompt for coder
func buildTaskPrompt(task Task) string {
	var sb strings.Builder
	sb.WriteString("## Task\n\n")
	sb.WriteString(task.Description)
	sb.WriteString("\n\n")

	if len(task.Files) > 0 {
		sb.WriteString("## Files to work on\n\n")
		for _, f := range task.Files {
			sb.WriteString("- " + f + "\n")
		}
		sb.WriteString("\n")
	}

	if task.TestCriteria != "" {
		sb.WriteString("## Test criteria\n\n")
		sb.WriteString(task.TestCriteria)
		sb.WriteString("\n")
	}

	return sb.String()
}

// buildCriticPrompt builds the prompt for critic
func buildCriticPrompt(task Task, coderOutput *CoderOutput) string {
	var sb strings.Builder
	sb.WriteString("## Task that was implemented\n\n")
	sb.WriteString(task.Description)
	sb.WriteString("\n\n")

	sb.WriteString("## Files changed\n\n")
	for _, f := range coderOutput.FilesChanged {
		sb.WriteString(fmt.Sprintf("- %s (%s): %s\n", f.Path, f.Action, f.Description))
	}
	sb.WriteString("\n")

	sb.WriteString("## Test criteria\n\n")
	sb.WriteString(task.TestCriteria)
	sb.WriteString("\n\n")

	sb.WriteString("Please review the code changes and provide your verdict.\n")

	return sb.String()
}
