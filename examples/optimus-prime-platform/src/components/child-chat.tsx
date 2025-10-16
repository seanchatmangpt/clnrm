'use client';

import { useState, useEffect } from 'react';
import { useChat } from 'ai/react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Badge } from '@/components/ui/badge';
import { Message, detectVirtue, REWARD_URLS, PREMIUM_CTA_VARIANTS } from '@/lib/types';
import { trackEvent, getABVariant, trackPremiumView, trackPremiumClick } from '@/lib/telemetry';

export function ChildChat() {
  const [virtue, setVirtue] = useState<string>('');
  const [rewardUrl, setRewardUrl] = useState<string>('');
  const [premiumTitle, setPremiumTitle] = useState<string>('');
  const [premiumLink, setPremiumLink] = useState<string>('');
  const [abVariant, setAbVariant] = useState<'A' | 'B'>('A');

  const { messages, input, handleInputChange, handleSubmit, isLoading } = useChat({
    api: '/api/chat',
    body: { mode: 'child' },
    onResponse: (response) => {
      // Extract headers from response
      const virtueHeader = response.headers.get('X-Virtue');
      const rewardHeader = response.headers.get('X-Reward-Url');
      const premiumTitleHeader = response.headers.get('X-Premium-Title');
      const premiumLinkHeader = response.headers.get('X-Premium-Link');

      if (virtueHeader) setVirtue(virtueHeader);
      if (rewardHeader) setRewardUrl(rewardHeader);
      if (premiumTitleHeader) setPremiumTitle(premiumTitleHeader);
      if (premiumLinkHeader) setPremiumLink(premiumLinkHeader);
    },
  });

  useEffect(() => {
    // Initialize A/B variant and track session start
    const variant = getABVariant();
    setAbVariant(variant);
    trackEvent('session_start', { mode: 'child', variant });
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
    trackEvent('reward_click', { virtue, variant: abVariant });
  };

  const handlePremiumClick = () => {
    trackPremiumClick(abVariant);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <Card className="child-panel">
        <CardHeader>
          <CardTitle className="text-2xl text-[hsl(var(--autobot-red))] flex items-center gap-3">
            <div className="w-8 h-8 bg-[hsl(var(--autobot-red))] rounded-full flex items-center justify-center text-white font-bold">
              O
            </div>
            Optimus Prime
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-[hsl(var(--gunmetal))]">
            Share your achievements and let Optimus Prime recognize your leadership qualities!
          </p>
        </CardContent>
      </Card>

      {/* Chat Messages */}
      <div className="space-y-4">
        {messages.map((message) => (
          <Card key={message.id} className={`${
            message.role === 'user'
              ? 'child-panel ml-12'
              : 'executive-panel mr-12'
          }`}>
            <CardContent className="p-4">
              <div className={`font-medium mb-2 ${
                message.role === 'user'
                  ? 'text-[hsl(var(--autobot-red))]'
                  : 'text-[hsl(var(--cyber-blue))]'
              }`}>
                {message.role === 'user' ? 'You' : 'Optimus Prime'}
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
              {isLoading ? 'Sending...' : 'Send'}
            </Button>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
