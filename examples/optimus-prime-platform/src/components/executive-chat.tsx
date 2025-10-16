'use client';

import { useState, useEffect } from 'react';
import { useChat } from 'ai/react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Message } from '@/lib/types';
import { trackEvent } from '@/lib/telemetry';

export function ExecutiveChat() {
  const { messages, input, handleInputChange, handleSubmit, isLoading } = useChat({
    api: '/api/chat',
    body: { mode: 'executive' },
  });

  useEffect(() => {
    // Track session start
    trackEvent('session_start', { mode: 'executive' });
  }, []);

  const handleFormSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    handleSubmit(e);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="executive-panel">
        <CardHeader>
          <CardTitle className="text-2xl text-[hsl(var(--cyber-blue))] flex items-center gap-3">
            <div className="w-8 h-8 bg-[hsl(var(--cyber-blue))] rounded-full flex items-center justify-center text-white font-bold">
              A
            </div>
            Executive Analytics
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-[hsl(var(--gunmetal))]">
            Ask questions about KPIs, revenue, and performance metrics. Get real-time insights from our analytics engine.
          </p>
        </CardContent>
      </Card>

      {/* Chat Messages */}
      <div className="space-y-4">
        {messages.map((message) => (
          <Card key={message.id} className={`${
            message.role === 'user'
              ? 'executive-panel ml-12'
              : 'bg-gradient-to-r from-[hsl(var(--cyber-blue))]/10 to-[hsl(var(--steel))]/10 mr-12 border-[hsl(var(--cyber-blue))]/30'
          }`}>
            <CardContent className="p-4">
              <div className={`font-medium mb-2 ${
                message.role === 'user'
                  ? 'text-[hsl(var(--cyber-blue))]'
                  : 'text-[hsl(var(--gunmetal))]'
              }`}>
                {message.role === 'user' ? 'You' : 'Analytics Engine'}
              </div>
              <div className="text-[hsl(var(--gunmetal))]">
                {message.content}
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Input Form */}
      <Card className="executive-panel">
        <CardContent className="p-4">
          <form onSubmit={handleFormSubmit} className="flex gap-2">
            <Input
              value={input}
              onChange={handleInputChange}
              placeholder="Ask about KPIs, revenue, or performance metrics..."
              className="flex-1 border-[hsl(var(--cyber-blue))]/30 focus:border-[hsl(var(--cyber-blue))]"
              disabled={isLoading}
            />
            <Button
              type="submit"
              disabled={isLoading || !input.trim()}
              className="cyber-button"
            >
              {isLoading ? 'Analyzing...' : 'Ask'}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
