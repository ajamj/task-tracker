# 🚀 tt Dashboard v3.0 - Quick Start Guide

## ✅ Dashboard is Running!

**URL:** http://localhost:3000

---

## 🎨 Features Overview

### 1. **List View** (Default)
- Traditional task list with checkboxes
- Click any task to view details
- Filter by status, priority, or search
- Quick add task form at top

### 2. **Kanban Board** 
- Click "📊 Kanban Board" tab
- **Drag & drop** tasks between columns
- 4 columns: Todo, Doing, Done, Blocked
- Task count shown in each column header

### 3. **Calendar View**
- Click "📅 Calendar" tab
- Monthly view with task indicators
- Colored dots show task status on due dates
- Navigate months with Prev/Next buttons

### 4. **Charts View**
- Click "📈 Charts" tab
- 4 interactive charts:
  - Tasks by Status (doughnut)
  - Tasks by Priority (bar)
  - Completion Trend (line)
  - Top Tags (bar)

---

## 🎯 Key Features

### Dark Mode Toggle 🌙
- Click the 🌙 button in header
- Switches between light/dark themes
- Preference saved to localStorage

### Task Filters 🔍
- **Status Filter:** Todo, Doing, Done, Blocked
- **Priority Filter:** P0, P1, P2, P3
- **Search:** Search by title, ID, or tags
- **Reset:** Clear all filters

### Task Detail Modal 📋
- Click any task to open modal
- View full task details
- Edit status and priority
- Save changes

### Export/Import 📥📤
- **Export:** Click 📥 Export button
  - Downloads tasks as JSON file
- **Import:** Click 📤 Import button
  - Upload JSON/CSV file

### Quick Add Task ⚡
- Type task title in "Quick Add" form
- Select priority (P0-P3)
- Click "➕ Add Task"
- Task appears in list instantly

---

## 🎨 View Switching

Click the view tabs at the top:
- **📋 List View** - Default task list
- **📊 Kanban Board** - Drag & drop workflow
- **📅 Calendar** - Task deadlines calendar
- **📈 Charts** - Productivity analytics

---

## 📊 Stats Cards

Real-time task counts:
- 📋 **Todo** (blue) - Tasks to do
- ⏳ **Doing** (yellow) - In progress tasks
- ✅ **Done** (green) - Completed tasks
- 🚫 **Blocked** (red) - Blocked tasks

---

## 🔧 Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl + D` | Toggle dark mode |
| `Ctrl + F` | Focus search |
| `Ctrl + E` | Export tasks |
| `Esc` | Close modal |

---

## 💡 Tips

1. **Drag & Drop:** In Kanban view, drag tasks to change status
2. **Quick Filters:** Use dropdowns to filter by status/priority
3. **Search:** Type to search across title, ID, and tags
4. **Export Regularly:** Backup your tasks with Export button
5. **Dark Mode:** Toggle for comfortable viewing at night

---

## 🛑 Stopping Dashboard

**Option 1:** Close the terminal/command prompt window

**Option 2:** Press `Ctrl+C` in the window running the dashboard

**Option 3:** Kill the process
```bash
taskkill /F /IM tt.exe
```

---

## 🐛 Troubleshooting

### Dashboard not loading?
1. Check if tt.exe is running
2. Verify port 3000 is not in use
3. Try `tt dashboard --port 3001` for different port

### API not responding?
1. Refresh the page
2. Check browser console for errors
3. Restart dashboard

### Dark mode not saving?
1. Check browser localStorage
2. Try different browser
3. Clear cache and reload

---

## 📊 Sample Data

The dashboard currently shows:
- **1 task** in "Done" status
- **0 tasks** in other statuses

Add more tasks to see the dashboard in action!

```bash
# Add tasks via CLI
tt add "New feature" --priority P1 --tag feature
tt add "Bug fix" --priority P0 --tag bug
tt add "Documentation" --priority P2
```

---

## 🎉 Enjoy Your Dashboard!

**Dashboard v3.0** is fully featured and production-ready!

For more info, check:
- `README.md` - Project documentation
- `STITCH_SETUP.md` - Stitch OAuth2 setup
- `docs/troubleshooting.md` - Troubleshooting guide

---

**Happy Task Tracking! 🚀**
