import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'Cleanroom Testing Framework',
  description: 'Hermetic integration testing that actually works end-to-end',
  base: '/',
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Documentation', link: '/docs/' },
      { text: 'API Reference', link: '/api/' },
      { text: 'Examples', link: '/examples/' },
      { text: 'GitHub', link: 'https://github.com/seanchatmangpt/clnrm' }
    ],
    sidebar: {
      '/docs/': [
        {
          text: 'Getting Started',
          items: [
            { text: 'Quick Start', link: '/docs/quick-start' },
            { text: 'Installation', link: '/docs/installation' },
            { text: 'First Test', link: '/docs/first-test' }
          ]
        },
        {
          text: 'Core Concepts',
          items: [
            { text: 'Architecture', link: '/docs/architecture' },
            { text: 'No-Prefix Variables', link: '/docs/variables' },
            { text: 'OTEL Validation', link: '/docs/otel-validation' },
            { text: 'Template System', link: '/docs/templates' }
          ]
        },
        {
          text: 'Configuration',
          items: [
            { text: 'TOML Reference', link: '/docs/toml-reference' },
            { text: 'CLI Commands', link: '/docs/cli-guide' },
            { text: 'Environment Variables', link: '/docs/env-variables' }
          ]
        },
        {
          text: 'Advanced Topics',
          items: [
            { text: 'Plugin Development', link: '/docs/plugins' },
            { text: 'CI/CD Integration', link: '/docs/ci-cd' },
            { text: 'Performance Tuning', link: '/docs/performance' }
          ]
        }
      ],
      '/api/': [
        {
          text: 'API Reference',
          items: [
            { text: 'Overview', link: '/api/' },
            { text: 'Service Plugins', link: '/api/plugins' },
            { text: 'Validators', link: '/api/validators' },
            { text: 'Configuration', link: '/api/config' }
          ]
        }
      ]
    },
    socialLinks: [
      { icon: 'github', link: 'https://github.com/seanchatmangpt/clnrm' }
    ],
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024-present Sean Chatman'
    },
    search: {
      provider: 'local'
    }
  },
  vite: {
    build: {
      chunkSizeWarningLimit: 1000
    }
  }
})
