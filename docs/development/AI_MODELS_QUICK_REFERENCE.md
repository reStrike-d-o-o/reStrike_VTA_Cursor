# AI Models Quick Reference

Quick reference for selecting the best AI model for specific reStrike VTA development tasks.

> **Note**: Based on Windows-only reStrike VTA project (commit `4d222ceed0cd698b7e3ba0d7037f51388d553803`)

---

## 🚀 **Task-Based AI Model Selection**

### **🏆 Best Performance (Unlimited Budget)**

| Task | Primary | Secondary | Cost |
|------|---------|-----------|------|
| **System Architecture** | Claude 3 Opus | Claude 3.5 Sonnet | $20-35/month |
| **Critical Bug Fixes** | Claude 3 Opus | Claude 3.5 Sonnet | $20-35/month |
| **Performance Optimization** | Claude 3.5 Sonnet | GPT-4 Turbo | $35/month |
| **Security Review** | Claude 3 Opus | Claude 3.5 Sonnet | $20-35/month |
| **Production Deployment** | Claude 3.5 Sonnet | Claude 3 Opus | $35/month |

### **💰 Best Budget (Cost-Conscious)**

| Task | Primary | Secondary | Cost |
|------|---------|-----------|------|
| **Daily Development** | Claude 3 Haiku | Gemini Pro | $0-10/month |
| **Code Review** | Claude 3 Haiku | GPT-3.5 Turbo | $0-10/month |
| **Documentation** | Claude 3 Haiku | Gemini Pro | $0-5/month |
| **Simple Bug Fixes** | Claude 3 Haiku | GPT-3.5 Turbo | $0-10/month |
| **Learning** | Gemini Pro | Claude 3 Haiku | $0/month |

### **⚖️ Balanced Approach (Recommended)**

| Task | Primary | Secondary | Cost |
|------|---------|-----------|------|
| **Complex Features** | Claude 3.5 Sonnet | Claude 3 Haiku | $15-20/month |
| **Code Generation** | GPT-4 Turbo | Claude 3 Haiku | $20-25/month |
| **Testing** | GPT-4 Turbo | Claude 3 Haiku | $20-25/month |
| **UI/UX** | Claude 3.5 Sonnet | GPT-4 Turbo | $35/month |
| **Integration** | Claude 3.5 Sonnet | Claude 3 Haiku | $15-20/month |

---

## 🎯 **Project-Specific Recommendations**

### **Rust Backend Development (1,663 lines)**
```bash
# Critical Tasks (Performance)
- Architecture: Claude 3 Opus
- Optimization: Claude 3.5 Sonnet
- Security: Claude 3 Opus

# Regular Tasks (Budget)
- Code Review: Claude 3 Haiku
- Bug Fixes: Claude 3 Haiku
- Documentation: Claude 3 Haiku
```

### **React Frontend Development (2,000+ lines)**
```bash
# Critical Tasks (Performance)
- Component Architecture: Claude 3.5 Sonnet
- Performance: Claude 3.5 Sonnet
- State Management: Claude 3.5 Sonnet

# Regular Tasks (Budget)
- Component Development: Claude 3 Haiku
- UI/UX: GPT-3.5 Turbo
- Testing: Claude 3 Haiku
```

### **Real-time Systems Integration**
```bash
# Critical Tasks (Performance)
- OBS WebSocket: Claude 3 Opus
- PSS Protocol: Claude 3 Opus
- Video Processing: Claude 3.5 Sonnet

# Regular Tasks (Budget)
- Basic Integration: Claude 3 Haiku
- Debugging: Claude 3 Haiku
- Documentation: Gemini Pro
```

---

## 💡 **Quick Decision Matrix**

### **Choose Claude 3 Opus When:**
- ✅ System architecture decisions
- ✅ Critical performance optimization
- ✅ Security audits
- ✅ Complex real-time integration
- ✅ Production deployment planning

### **Choose Claude 3.5 Sonnet When:**
- ✅ Complex feature development
- ✅ Code optimization
- ✅ Architecture review
- ✅ Performance tuning
- ✅ Professional documentation

### **Choose GPT-4 Turbo When:**
- ✅ Rapid prototyping
- ✅ Code generation
- ✅ Testing implementation
- ✅ UI/UX development
- ✅ API design

### **Choose Claude 3 Haiku When:**
- ✅ Daily development tasks
- ✅ Code review
- ✅ Simple bug fixes
- ✅ Documentation
- ✅ Learning new concepts

### **Choose GPT-3.5 Turbo When:**
- ✅ Basic coding tasks
- ✅ Code explanation
- ✅ Simple refactoring
- ✅ Basic documentation
- ✅ Learning patterns

### **Choose Gemini Pro When:**
- ✅ Free development
- ✅ Basic code analysis
- ✅ Integration help
- ✅ Documentation
- ✅ Learning concepts

---

## 📊 **Cost Comparison**

### **Monthly Budget Scenarios**

#### **Professional ($55/month)**
- Claude 3.5 Sonnet: $15
- GPT-4 Turbo: $20
- Claude 3 Opus: $20
- **Best for**: Production development

#### **Standard ($30/month)**
- Claude 3.5 Sonnet: $15
- Claude 3 Haiku: $10
- GPT-3.5 Turbo: $5
- **Best for**: Regular development

#### **Budget ($10/month)**
- Claude 3 Haiku: $10
- Gemini Pro: Free
- GPT-3.5 Turbo: Free tier
- **Best for**: Learning and basic development

#### **Free ($0/month)**
- Claude 3 Haiku: Free tier
- Gemini Pro: Free tier
- GPT-3.5 Turbo: Free tier
- **Best for**: Exploration and learning

---

## 🎯 **Recommended Stacks by Use Case**

### **For reStrike VTA Development**

#### **Production Development**
```bash
Primary: Claude 3.5 Sonnet ($15/month)
Secondary: GPT-4 Turbo ($20/month)
Critical: Claude 3 Opus ($20/month)
Total: $55/month
```

#### **Feature Development**
```bash
Primary: Claude 3.5 Sonnet ($15/month)
Secondary: Claude 3 Haiku ($10/month)
Total: $25/month
```

#### **Learning & Exploration**
```bash
Primary: Claude 3 Haiku (Free tier)
Secondary: Gemini Pro (Free tier)
Backup: GPT-3.5 Turbo (Free tier)
Total: $0/month
```

---

## 🚀 **Quick Start Commands**

### **For Critical Development**
```bash
# Use Claude 3.5 Sonnet for:
- Complex Rust plugin development
- React component optimization
- Tauri integration issues
- Performance optimization

# Use Claude 3 Opus for:
- System architecture decisions
- Security reviews
- Production deployment
```

### **For Daily Development**
```bash
# Use Claude 3 Haiku for:
- Regular coding tasks
- Code review
- Documentation
- Simple bug fixes

# Use GPT-3.5 Turbo for:
- Quick code generation
- Basic debugging
- Simple refactoring
```

### **For Budget Development**
```bash
# Use free tiers:
- Claude 3 Haiku (free tier)
- Gemini Pro (free tier)
- GPT-3.5 Turbo (free tier)

# Upgrade only when needed:
- Claude 3.5 Sonnet for complex tasks
- GPT-4 Turbo for code generation
```

---

## 📚 **Resources**

### **AI Model Access**
- [Claude (Anthropic)](https://claude.ai/)
- [ChatGPT (OpenAI)](https://chat.openai.com/)
- [Gemini (Google)](https://aistudio.google.com/)

### **Project Documentation**
- [AI Models Guide](./AI_MODELS_GUIDE.md)
- [Windows-Only Conversion Guide](./WINDOWS_ONLY_CONVERSION_GUIDE.md)
- [VSCode Setup Guide](./VSCODE_WINDOWS_SETUP.md)

---

**🎯 This quick reference provides instant AI model selection for any reStrike VTA development task, optimizing for both performance and budget.** 