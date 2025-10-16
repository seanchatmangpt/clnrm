"use client";

import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { Message } from "@/lib/types";
import {
  trackEvent,
  getABVariant,
  trackPremiumView,
  trackPremiumClick,
  getVirtueCount,
  getVirtueHistory,
  trackRewardView,
} from "@/lib/telemetry";
import { VirtueHistory } from "@/lib/types";

export function ChildChat() {
  const [virtue, setVirtue] = useState<string>("");
  const [rewardUrl, setRewardUrl] = useState<string>("");
  const [premiumTitle, setPremiumTitle] = useState<string>("");
  const [premiumLink, setPremiumLink] = useState<string>("");
  const [abVariant, setAbVariant] = useState<"A" | "B">("A");

  const [messages, setMessages] = useState<Message[]>([]);
  const [input, setInput] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  // Virtue tracking state
  const [virtueCount, setVirtueCount] = useState<Record<string, number>>({});
  const [virtueHistoryList, setVirtueHistoryList] = useState<VirtueHistory[]>(
    []
  );
  const [showHistory, setShowHistory] = useState(false);

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    if (!input.trim()) return;

    const userMessage: Message = {
      id: crypto.randomUUID(),
      role: "user",
      content: input,
      timestamp: Date.now(),
    };

    setMessages((prev) => [...prev, userMessage]);
    setIsLoading(true);
    setInput("");

    try {
      const response = await fetch("/api/chat", {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          mode: "child",
          messages: [...messages, userMessage],
        }),
      });

      if (!response.ok) {
        throw new Error("Failed to send message");
      }

      // Extract headers from response
      const virtueHeader = response.headers.get("X-Virtue");
      const rewardHeader = response.headers.get("X-Reward-Url");
      const premiumTitleHeader = response.headers.get("X-Premium-Title");
      const premiumLinkHeader = response.headers.get("X-Premium-Link");

      if (virtueHeader) {
        setVirtue(virtueHeader);
        // Update virtue tracking
        setVirtueCount(getVirtueCount());
        setVirtueHistoryList(getVirtueHistory());
      }
      if (rewardHeader) {
        setRewardUrl(rewardHeader);
        // Track reward view when it appears
        if (virtueHeader) {
          trackRewardView(virtueHeader, abVariant);
        }
      }
      if (premiumTitleHeader) setPremiumTitle(premiumTitleHeader);
      if (premiumLinkHeader) setPremiumLink(premiumLinkHeader);

      // Read streaming response
      const reader = response.body?.getReader();
      const decoder = new TextDecoder();

      const assistantMessage: Message = {
        id: crypto.randomUUID(),
        role: "assistant",
        content: "",
        timestamp: Date.now(),
      };

      setMessages((prev) => [...prev, assistantMessage]);

      if (reader) {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          // Parse Ollama streaming format
          const lines = chunk.split("\n").filter((line) => line.trim());
          for (const line of lines) {
            try {
              const data = JSON.parse(line);
              if (data.response) {
                assistantMessage.content += data.response;
                setMessages((prev) =>
                  prev.map((msg) =>
                    msg.id === assistantMessage.id
                      ? { ...msg, content: assistantMessage.content }
                      : msg
                  )
                );
              }
            } catch (e) {
              // Ignore parsing errors for non-JSON lines
            }
          }
        }
      }
    } catch (error) {
      console.error("Error sending message:", error);
      const errorMessage: Message = {
        id: crypto.randomUUID(),
        role: "assistant",
        content: "Sorry, I encountered an error. Please try again.",
        timestamp: Date.now(),
      };
      setMessages((prev) => [...prev, errorMessage]);
    } finally {
      setIsLoading(false);
    }
  };

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setInput(e.target.value);
  };

  useEffect(() => {
    // Initialize A/B variant and track session start
    const variant = getABVariant();
    setAbVariant(variant);
    trackEvent("session_start", { mode: "child", variant });

    // Load virtue history on mount
    setVirtueCount(getVirtueCount());
    setVirtueHistoryList(getVirtueHistory());
  }, []);

  const handleFormSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    handleSubmit(e);

    // Track premium view when submitting
    if (premiumTitle && premiumLink) {
      trackPremiumView(abVariant);
    }
  };

  const handleRewardClick = () => {
    trackEvent("reward_click", { virtue, variant: abVariant });
  };

  const handlePremiumClick = () => {
    trackPremiumClick(abVariant);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="child-panel">
        <CardHeader>
          <CardTitle className="text-2xl text-[hsl(var(--autobot-red))] flex items-center justify-between">
            <div className="flex items-center gap-3">
              <div className="w-8 h-8 bg-[hsl(var(--autobot-red))] rounded-full flex items-center justify-center text-white font-bold">
                O
              </div>
              Optimus Prime
            </div>
            {/* Virtue Counter Badge */}
            {Object.keys(virtueCount).length > 0 && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => setShowHistory(!showHistory)}
                className="text-xs"
              >
                Virtues: {Object.values(virtueCount).reduce((a, b) => a + b, 0)}
              </Button>
            )}
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-[hsl(var(--gunmetal))]">
            Share your achievements and let Optimus Prime recognize your
            leadership qualities!
          </p>

          {/* Virtue Counter Details */}
          {Object.keys(virtueCount).length > 0 && (
            <div className="mt-4 flex flex-wrap gap-2">
              {Object.entries(virtueCount).map(([virtue, count]) => (
                <Badge
                  key={virtue}
                  variant="secondary"
                  className="bg-[hsl(var(--energon))]/20 text-[hsl(var(--gunmetal))]"
                >
                  {virtue.charAt(0).toUpperCase() + virtue.slice(1)}: {count}
                </Badge>
              ))}
            </div>
          )}
        </CardContent>
      </Card>

      {/* Virtue History Panel */}
      {showHistory && virtueHistoryList.length > 0 && (
        <Card className="bg-gradient-to-r from-[hsl(var(--energon))]/10 to-[hsl(var(--autobot-red))]/10 border-[hsl(var(--energon))]">
          <CardHeader>
            <CardTitle className="text-lg text-[hsl(var(--autobot-red))]">
              Your Leadership Journey
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-3 max-h-60 overflow-y-auto">
              {virtueHistoryList
                .slice()
                .reverse()
                .map((entry) => (
                  <div
                    key={entry.id}
                    className="flex items-start gap-3 p-3 bg-white/50 rounded-md"
                  >
                    <Badge className="bg-[hsl(var(--energon))] text-[hsl(var(--gunmetal))] font-semibold shrink-0">
                      {entry.virtue.charAt(0).toUpperCase() +
                        entry.virtue.slice(1)}
                    </Badge>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm text-[hsl(var(--gunmetal))]">
                        {entry.achievement}
                      </p>
                      <p className="text-xs text-[hsl(var(--steel))] mt-1">
                        {new Date(entry.timestamp).toLocaleString()}
                      </p>
                    </div>
                  </div>
                ))}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Chat Messages */}
      <div className="space-y-4">
        {messages.map((message) => (
          <Card
            key={message.id}
            className={`${
              message.role === "user"
                ? "child-panel ml-12"
                : "executive-panel mr-12"
            }`}
          >
            <CardContent className="p-4">
              <div
                className={`font-medium mb-2 ${
                  message.role === "user"
                    ? "text-[hsl(var(--autobot-red))]"
                    : "text-[hsl(var(--cyber-blue))]"
                }`}
              >
                {message.role === "user" ? "You" : "Optimus Prime"}
              </div>
              <div className="text-[hsl(var(--gunmetal))]">
                {message.content}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Virtue Badge */}
      {virtue && (
        <Card className="bg-gradient-to-r from-[hsl(var(--energon))]/20 to-[hsl(var(--autobot-red))]/20 border-[hsl(var(--energon))]">
          <CardContent className="p-4">
            <div className="flex items-center gap-3">
              <Badge className="bg-[hsl(var(--energon))] text-[hsl(var(--gunmetal))] font-semibold">
                {virtue.charAt(0).toUpperCase() + virtue.slice(1)}
              </Badge>
              <span className="text-[hsl(var(--gunmetal))]">
                Recognized for demonstrating leadership!
              </span>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Reward Section */}
      {rewardUrl && (
        <Card className="child-panel">
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-semibold text-[hsl(var(--autobot-red))]">
                  🎉 Achievement Unlocked!
                </h3>
                <p className="text-[hsl(var(--gunmetal))] text-sm">
                  You've earned a special reward for your {virtue}!
                </p>
              </div>
              <Button
                onClick={handleRewardClick}
                className="autobot-button"
                asChild
              >
                <a href={rewardUrl} target="_blank" rel="noopener noreferrer">
                  Claim Reward
                </a>
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Premium CTA */}
      {premiumTitle && premiumLink && (
        <Card className="bg-gradient-to-r from-[hsl(var(--cyber-blue))]/20 to-[hsl(var(--energon))]/20 border-[hsl(var(--cyber-blue))]">
          <CardContent className="p-4">
            <div className="flex items-center justify-between">
              <div>
                <h3 className="font-semibold text-[hsl(var(--cyber-blue))]">
                  🚀 {premiumTitle}
                </h3>
                <p className="text-[hsl(var(--gunmetal))] text-sm">
                  Unlock exclusive adventures and premium features!
                </p>
              </div>
              <Button
                onClick={handlePremiumClick}
                className="cyber-button"
                asChild
              >
                <a href={premiumLink} target="_blank" rel="noopener noreferrer">
                  Upgrade Now
                </a>
              </Button>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Input Form */}
      <Card className="child-panel">
        <CardContent className="p-4">
          <form onSubmit={handleFormSubmit} className="flex gap-2">
            <Input
              value={input}
              onChange={handleInputChange}
              placeholder="Share your achievement or ask Optimus Prime..."
              className="flex-1 border-[hsl(var(--autobot-red))]/30 focus:border-[hsl(var(--autobot-red))]"
              disabled={isLoading}
            />
            <Button
              type="submit"
              disabled={isLoading || !input.trim()}
              className="autobot-button"
            >
              {isLoading ? "Sending..." : "Send"}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
