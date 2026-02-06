package main

import (
	"context"
	"embed"
	"fmt"
	"os"

	agent "github.com/MoonshotAI/kimi-agent-sdk/go"
	"github.com/MoonshotAI/kimi-agent-sdk/go/wire"
)

//go:embed prompts/*.md
var promptFS embed.FS

// loadPrompt reads a prompt file from embedded filesystem
func loadPrompt(name string) string {
	data, err := promptFS.ReadFile("prompts/" + name + ".md")
	if err != nil {
		fmt.Fprintf(os.Stderr, "Failed to load prompt %s: %v\n", name, err)
		return ""
	}
	return string(data)
}

// Agent wraps Kimi SDK session with a specific role
type Agent struct {
	role      string
	session   *agent.Session
	workDir   string
	sessionID string
}

// NewAgent creates a new agent with the given role
func NewAgent(role, workDir, sessionID string) (*Agent, error) {
	session, err := agent.NewSession(
		agent.WithWorkDir(workDir),
		agent.WithAutoApprove(),
		agent.WithSession(sessionID),
	)
	if err != nil {
		return nil, fmt.Errorf("failed to create session: %w", err)
	}

	return &Agent{
		role:      role,
		session:   session,
		workDir:   workDir,
		sessionID: sessionID,
	}, nil
}

// Close closes the agent session
func (a *Agent) Close() {
	if a.session != nil {
		a.session.Close()
	}
}

// Run executes a prompt and returns the response
func (a *Agent) Run(ctx context.Context, userPrompt string) (string, error) {
	// Combine system prompt with user prompt
	systemPrompt := loadPrompt(a.role)
	fullPrompt := systemPrompt + "\n\n---\n\n" + userPrompt

	turn, err := a.session.Prompt(ctx, wire.NewStringContent(fullPrompt))
	if err != nil {
		return "", fmt.Errorf("prompt failed: %w", err)
	}

	var response string

	// Consume all messages
	for step := range turn.Steps {
		for msg := range step.Messages {
			switch m := msg.(type) {
			case wire.ApprovalRequest:
				_ = m.Respond(wire.ApprovalRequestResponseApprove)
			case wire.ContentPart:
				if m.Type == wire.ContentPartTypeText && m.Text.Valid {
					response += m.Text.Value
				}
			}
		}
	}

	if err := turn.Err(); err != nil {
		return "", fmt.Errorf("turn error: %w", err)
	}

	return response, nil
}

// RunWithCallback executes a prompt with streaming callback
func (a *Agent) RunWithCallback(ctx context.Context, userPrompt string, onText func(string)) error {
	systemPrompt := loadPrompt(a.role)
	fullPrompt := systemPrompt + "\n\n---\n\n" + userPrompt

	turn, err := a.session.Prompt(ctx, wire.NewStringContent(fullPrompt))
	if err != nil {
		return fmt.Errorf("prompt failed: %w", err)
	}

	for step := range turn.Steps {
		for msg := range step.Messages {
			switch m := msg.(type) {
			case wire.ApprovalRequest:
				_ = m.Respond(wire.ApprovalRequestResponseApprove)
			case wire.ContentPart:
				if m.Type == wire.ContentPartTypeText && m.Text.Valid {
					if onText != nil {
						onText(m.Text.Value)
					}
				}
			}
		}
	}

	return turn.Err()
}
