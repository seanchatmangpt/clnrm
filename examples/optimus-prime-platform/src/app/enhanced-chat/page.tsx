'use client';

import { useState, useRef } from 'react';
import {
  PromptInput,
  PromptInputActionAddAttachments,
  PromptInputActionMenu,
  PromptInputActionMenuContent,
  PromptInputActionMenuTrigger,
  PromptInputAttachment,
  PromptInputAttachments,
  PromptInputBody,
  type PromptInputMessage,
  PromptInputSubmit,
  PromptInputTextarea,
  PromptInputFooter,
  PromptInputTools,
} from '@/components/ai-elements/prompt-input';
import {
  Conversation,
  ConversationContent,
  ConversationScrollButton,
} from '@/components/ai-elements/conversation';
import { Message, MessageContent } from '@/components/ai-elements/message';
import { Response } from '@/components/ai-elements/response';
import {
  ChainOfThought,
  ChainOfThoughtContent,
  ChainOfThoughtHeader,
  ChainOfThoughtStep,
} from '@/components/ai-elements/chain-of-thought';
import { SparklesIcon, BrainIcon } from 'lucide-react';

interface ChatMessage {
  id: string;
  role: 'user' | 'assistant';
  content: string;
  thinking?: {
    virtue?: string;
    reasoning?: string;
  };
}

export default function EnhancedChatPage() {
  const [text, setText] = useState<string>('');
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [status, setStatus] = useState<'submitted' | 'streaming' | 'ready' | 'error'>('ready');
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  const handleSubmit = async (message: PromptInputMessage) => {
    const hasText = Boolean(message.text);
    const hasAttachments = Boolean(message.files?.length);

    if (!(hasText || hasAttachments)) {
      return;
    }

    setStatus('submitted');

    const userMessage: ChatMessage = {
      id: crypto.randomUUID(),
      role: 'user',
      content: message.text || 'Sent with attachments',
    };

    setMessages((prev) => [...prev, userMessage]);
    setText('');

    try {
      const response = await fetch('/api/chat', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          mode: 'child',
          messages: [{ role: 'user', content: message.text }],
        }),
      });

      if (!response.ok) {
        throw new Error('Failed to send message');
      }

      setStatus('streaming');

      // Get virtue from headers
      const virtue = response.headers.get('x-virtue');
      const rewardUrl = response.headers.get('x-reward-url');

      const reader = response.body?.getReader();
      const decoder = new TextDecoder();

      const assistantMessage: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: '',
        thinking: {
          virtue: virtue || undefined,
          reasoning: `Detected ${virtue} in your message. Preparing personalized response...`,
        },
      };

      setMessages((prev) => [...prev, assistantMessage]);

      if (reader) {
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;

          const chunk = decoder.decode(value);
          const lines = chunk.split('\n').filter((line) => line.trim());

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
              // Ignore parsing errors
            }
          }
        }
      }

      setStatus('ready');
    } catch (error) {
      console.error('Error sending message:', error);
      setStatus('error');
      const errorMessage: ChatMessage = {
        id: crypto.randomUUID(),
        role: 'assistant',
        content: 'Sorry, I encountered an error. Please try again.',
      };
      setMessages((prev) => [...prev, errorMessage]);
      setStatus('ready');
    }
  };

  return (
    <div className="container mx-auto p-6 h-screen flex flex-col">
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-[hsl(var(--cyber-blue))] flex items-center gap-3">
          <div className="w-10 h-10 bg-[hsl(var(--cyber-blue))] rounded-full flex items-center justify-center text-white">
            <SparklesIcon className="w-6 h-6" />
          </div>
          Optimus Prime Character Platform
        </h1>
        <p className="text-muted-foreground mt-2">
          Enhanced chat with AI Elements - Share your achievements and receive personalized guidance
        </p>
      </div>

      <div className="flex-1 flex flex-col border rounded-lg bg-background overflow-hidden">
        <Conversation>
          <ConversationContent>
            {messages.map((message) => (
              <div key={message.id} className="space-y-2">
                {message.thinking && message.role === 'assistant' && (
                  <ChainOfThought defaultOpen>
                    <ChainOfThoughtHeader>
                      <span className="flex items-center gap-2">
                        <BrainIcon className="w-4 h-4" />
                        Thinking Process
                      </span>
                    </ChainOfThoughtHeader>
                    <ChainOfThoughtContent>
                      <ChainOfThoughtStep
                        icon={SparklesIcon}
                        label="Virtue Detection"
                        status="complete"
                      >
                        <div className="text-sm text-muted-foreground">
                          Detected virtue: <span className="font-semibold text-primary">{message.thinking.virtue}</span>
                        </div>
                      </ChainOfThoughtStep>
                      <ChainOfThoughtStep
                        label={message.thinking.reasoning || 'Processing your message...'}
                        status="complete"
                      />
                    </ChainOfThoughtContent>
                  </ChainOfThought>
                )}
                <Message from={message.role}>
                  <MessageContent>
                    <Response>{message.content}</Response>
                  </MessageContent>
                </Message>
              </div>
            ))}
          </ConversationContent>
          <ConversationScrollButton />
        </Conversation>

        <div className="p-4 border-t">
          <PromptInput onSubmit={handleSubmit} globalDrop multiple>
            <PromptInputBody>
              <PromptInputAttachments>
                {(attachment) => <PromptInputAttachment data={attachment} />}
              </PromptInputAttachments>
              <PromptInputTextarea
                onChange={(e) => setText(e.target.value)}
                placeholder="Share an achievement (e.g., 'I helped my friend with homework')"
                ref={textareaRef}
                value={text}
              />
            </PromptInputBody>
            <PromptInputFooter>
              <PromptInputTools>
                <PromptInputActionMenu>
                  <PromptInputActionMenuTrigger />
                  <PromptInputActionMenuContent>
                    <PromptInputActionAddAttachments />
                  </PromptInputActionMenuContent>
                </PromptInputActionMenu>
              </PromptInputTools>
              <PromptInputSubmit status={status} />
            </PromptInputFooter>
          </PromptInput>
        </div>
      </div>
    </div>
  );
}
