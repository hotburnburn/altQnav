// altQnav - 工具导航
const { invoke } = window.__TAURI__.core;
const { getCurrentWindow } = window.__TAURI__.window;

let appList = []; // 存储应用列表信息

// 动态生成应用格子
function generateNavGrid(apps) {
  const navGrid = document.getElementById("navGrid");
  navGrid.innerHTML = ""; // 清空现有内容

  apps.forEach((app, index) => {
    const navItem = document.createElement("div");
    navItem.className = "nav-item";
    navItem.dataset.index = index;

    // 如果有图标，显示图标；否则显示文字名称
    if (app.icon_path) {
      const img = document.createElement("img");
      // 使用Tauri的convertFileSrc转换本地文件路径
      const convertedPath = window.__TAURI__.core.convertFileSrc(app.icon_path);
      img.src = convertedPath;
      img.alt = app.display_name;
      img.className = "app-icon";
      img.onerror = () => {
        // 加载失败时显示文字
        navItem.innerHTML = "";
        const span = document.createElement("span");
        span.textContent = app.display_name;
        navItem.appendChild(span);
      };
      navItem.appendChild(img);
    } else {
      const span = document.createElement("span");
      span.textContent = app.display_name;
      navItem.appendChild(span);
    }

    // 如果应用正在运行，添加运行状态类
    if (app.is_running) {
      navItem.classList.add("running");
    }

    // 存储进程名和启动路径到 data 属性
    navItem.dataset.processName = app.process_name;
    navItem.dataset.launchPath = app.launch_path;

    // 绑定点击事件
    navItem.addEventListener("click", async () => {
      await handleAppClick(index);
    });

    navGrid.appendChild(navItem);
  });

  // 调整窗口大小
  adjustWindowSize(apps.length);
}

// 根据应用数量调整窗口大小
async function adjustWindowSize(appCount) {
  const itemWidth = 80;
  const itemHeight = 80;
  const gapX = 12; // 列间距
  const gapY = 24; // 行间距（与CSS一致）
  const columns = 4; // 固定4列
  const indicatorSpace = 20; // 小灯占用的空间
  const paddingTop = 30; // 上内边距
  const paddingBottom = 30; // 下内边距
  const paddingLeft = 30; // 左内边距
  const paddingRight = 30; // 右内边距

  const rows = Math.ceil(appCount / columns);

  // 宽度 = 4列格子 + 3个列间距 + 左右内边距
  const width =
    columns * itemWidth + (columns - 1) * gapX + paddingLeft + paddingRight;

  // 高度 = 行数×格子高度 + (行数-1)×行间距 + 小灯空间 + 上下内边距
  const height =
    rows * itemHeight +
    (rows - 1) * gapY +
    indicatorSpace +
    paddingTop +
    paddingBottom;

  const currentWindow = getCurrentWindow();
  await currentWindow.setSize({
    type: "Logical",
    width: Math.floor(width),
    height: Math.floor(height),
  });
}

async function loadAppList() {
  try {
    const apps = await invoke("get_app_list");

    // 保存应用列表
    appList = apps;

    // 生成或更新应用格子
    generateNavGrid(apps);
  } catch (error) {
    console.error("加载应用列表失败:", error);
  }
}

async function handleAppClick(index) {
  const app = appList[index];
  if (!app) {
    console.error("未找到应用信息");
    return;
  }

  try {
    // launch_or_focus_app 内部会自己检测应用状态，不需要额外获取
    const result = await invoke("launch_or_focus_app", {
      processName: app.process_name,
      launchPath: app.launch_path,
    });

    // 操作成功后关闭窗口
    const currentWindow = getCurrentWindow();
    await currentWindow.hide();
  } catch (error) {
    console.error("操作失败:", error);
  }
}

// 键盘快捷键映射：qwer asdf zxcv 对应前12个应用
const keyToIndexMap = {
  q: 0,
  w: 1,
  e: 2,
  r: 3,
  a: 4,
  s: 5,
  d: 6,
  f: 7,
  z: 8,
  x: 9,
  c: 10,
  v: 11,
};

// 处理键盘事件
function handleKeyPress(event) {
  const key = event.key.toLowerCase();
  const index = keyToIndexMap[key];

  // ESC 键关闭窗口
  if (event.key === "Escape") {
    event.preventDefault();
    const currentWindow = getCurrentWindow();
    currentWindow.hide();
    return;
  }

  // 如果按键在映射表中，且应用列表中存在对应索引的应用
  if (index !== undefined && index < appList.length) {
    event.preventDefault(); // 阻止默认行为
    handleAppClick(index);
  }
}

window.addEventListener("DOMContentLoaded", async () => {
  // 初始加载应用列表
  await loadAppList();

  // 监听窗口获得焦点事件,每次显示时刷新状态
  const currentWindow = getCurrentWindow();
  await currentWindow.listen("tauri://focus", async () => {
    await loadAppList();

    // 确保鼠标指针正确响应，触发一次鼠标移动事件
    document.body.style.pointerEvents = "none";
    setTimeout(() => {
      document.body.style.pointerEvents = "auto";
    }, 10);
  });

  // 添加键盘事件监听
  document.addEventListener("keydown", handleKeyPress);
});
