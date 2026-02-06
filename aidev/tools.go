package main

import (
	"bufio"
	"encoding/json"
	"fmt"
	"os"
	"strings"
)

// PlannerOutput represents the structured output from planner
type PlannerOutput struct {
	Analysis        string             `json:"analysis"`
	Questions       []Question         `json:"questions"`
	ResourcesNeeded []Resource         `json:"resources_needed"`
	Tasks           []Task             `json:"tasks"`
}

// Question represents a question that needs user confirmation
type Question struct {
	ID         int    `json:"id"`
	Question   string `json:"question"`
	Suggestion string `json:"suggestion"`
	Reason     string `json:"reason"`
}

// Resource represents an external resource needed
type Resource struct {
	Name        string `json:"name"`
	Description string `json:"description"`
	Required    bool   `json:"required"`
}

// Task represents a subtask to execute
type Task struct {
	ID           string   `json:"id"`
	Description  string   `json:"description"`
	Files        []string `json:"files"`
	DependsOn    []string `json:"depends_on"`
	TestCriteria string   `json:"test_criteria"`
}

// CoderOutput represents the structured output from coder
type CoderOutput struct {
	Status       string        `json:"status"` // completed, blocked, needs_clarification
	FilesChanged []FileChange  `json:"files_changed"`
	IssuesFound  []Issue       `json:"issues_found"`
	Notes        string        `json:"notes"`
}

// FileChange represents a file modification
type FileChange struct {
	Path        string `json:"path"`
	Action      string `json:"action"` // create, modify, delete
	Description string `json:"description"`
}

// Issue represents a problem found during implementation
type Issue struct {
	Type        string `json:"type"` // bug, design_flaw, missing_requirement
	Description string `json:"description"`
	Suggestion  string `json:"suggestion"`
}

// CriticOutput represents the structured output from critic
type CriticOutput struct {
	Verdict   string        `json:"verdict"` // approved, needs_changes, rejected
	Issues    []ReviewIssue `json:"issues"`
	Checklist Checklist     `json:"checklist"`
	Summary   string        `json:"summary"`
}

// ReviewIssue represents an issue found during code review
type ReviewIssue struct {
	Severity    string `json:"severity"` // critical, major, minor, suggestion
	File        string `json:"file"`
	Line        int    `json:"line"`
	Category    string `json:"category"` // bug, security, performance, style, maintainability
	Description string `json:"description"`
	Suggestion  string `json:"suggestion"`
}

// Checklist represents the review checklist
type Checklist struct {
	RequirementsMet       bool `json:"requirements_met"`
	NoRegressions         bool `json:"no_regressions"`
	TestsAdequate         bool `json:"tests_adequate"`
	SecurityReviewed      bool `json:"security_reviewed"`
	ErrorHandlingComplete bool `json:"error_handling_complete"`
}

// extractJSON extracts JSON from a response that may contain markdown
func extractJSON(response string) string {
	// Try to find JSON in code blocks
	if start := strings.Index(response, "```json"); start != -1 {
		start += 7 // skip "```json"
		if end := strings.Index(response[start:], "```"); end != -1 {
			return strings.TrimSpace(response[start : start+end])
		}
	}
	// Try to find JSON in generic code blocks
	if start := strings.Index(response, "```"); start != -1 {
		start += 3
		if end := strings.Index(response[start:], "```"); end != -1 {
			return strings.TrimSpace(response[start : start+end])
		}
	}
	// Assume the whole response is JSON
	return strings.TrimSpace(response)
}

// parsePlannerOutput parses planner response into structured output
func parsePlannerOutput(response string) (*PlannerOutput, error) {
	jsonStr := extractJSON(response)
	var output PlannerOutput
	if err := json.Unmarshal([]byte(jsonStr), &output); err != nil {
		return nil, fmt.Errorf("failed to parse planner output: %w", err)
	}
	return &output, nil
}

// parseCoderOutput parses coder response into structured output
func parseCoderOutput(response string) (*CoderOutput, error) {
	jsonStr := extractJSON(response)
	var output CoderOutput
	if err := json.Unmarshal([]byte(jsonStr), &output); err != nil {
		return nil, fmt.Errorf("failed to parse coder output: %w", err)
	}
	return &output, nil
}

// parseCriticOutput parses critic response into structured output
func parseCriticOutput(response string) (*CriticOutput, error) {
	jsonStr := extractJSON(response)
	var output CriticOutput
	if err := json.Unmarshal([]byte(jsonStr), &output); err != nil {
		return nil, fmt.Errorf("failed to parse critic output: %w", err)
	}
	return &output, nil
}

// askUser prompts the user for input and returns the response
func askUser(prompt string) string {
	fmt.Print(prompt)
	reader := bufio.NewReader(os.Stdin)
	input, _ := reader.ReadString('\n')
	return strings.TrimSpace(input)
}

// printQuestions prints questions in a formatted way
func printQuestions(questions []Question) {
	if len(questions) == 0 {
		return
	}
	fmt.Println("\n📋 Questions that need your confirmation:")
	fmt.Println(strings.Repeat("-", 50))
	for _, q := range questions {
		fmt.Printf("%d️⃣  %s\n", q.ID, q.Question)
		if q.Suggestion != "" {
			fmt.Printf("   💡 Suggestion: %s\n", q.Suggestion)
		}
		if q.Reason != "" {
			fmt.Printf("   ℹ️  Reason: %s\n", q.Reason)
		}
		fmt.Println()
	}
}

// printResources prints required resources in a formatted way
func printResources(resources []Resource) {
	if len(resources) == 0 {
		return
	}
	fmt.Println("\n🔑 Required resources:")
	fmt.Println(strings.Repeat("-", 50))
	for _, r := range resources {
		required := ""
		if r.Required {
			required = " (required)"
		}
		fmt.Printf("• %s%s: %s\n", r.Name, required, r.Description)
	}
	fmt.Println()
}

// printTasks prints tasks in a formatted way
func printTasks(tasks []Task) {
	if len(tasks) == 0 {
		return
	}
	fmt.Println("\n📝 Tasks to execute:")
	fmt.Println(strings.Repeat("-", 50))
	for _, t := range tasks {
		deps := ""
		if len(t.DependsOn) > 0 {
			deps = fmt.Sprintf(" (depends on: %s)", strings.Join(t.DependsOn, ", "))
		}
		fmt.Printf("[%s]%s %s\n", t.ID, deps, t.Description)
		if len(t.Files) > 0 {
			fmt.Printf("     Files: %s\n", strings.Join(t.Files, ", "))
		}
	}
	fmt.Println()
}
