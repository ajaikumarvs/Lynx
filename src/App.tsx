// src/App.tsx
import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { ArrowLeft, ArrowRight, RotateCcw, Home } from 'lucide-react'
import './App.css'

interface Tab {
  id: string;
  url: string;
  title: string;
  isActive: boolean;
  canGoBack: boolean;
  canGoForward: boolean;
}

function App() {
  const [tabs, setTabs] = useState<Tab[]>([]);
  const [currentUrl, setCurrentUrl] = useState('');
  const [activeTab, setActiveTab] = useState<Tab | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  useEffect(() => {
    // Listen for tab updates
    const unsubscribe = listen('tab-updated', (event: any) => {
      const { tab_id, title, url, can_go_back, can_go_forward } = event.payload;
      setTabs(prevTabs => 
        prevTabs.map(tab => 
          tab.id === tab_id 
            ? { 
                ...tab, 
                title, 
                url, 
                canGoBack: can_go_back, 
                canGoForward: can_go_forward 
              } 
            : tab
        )
      );
    });

    // Create initial tab
    handleNewTab();

    return () => {
      unsubscribe.then(fn => fn());
    };
  }, []);

  const handleNewTab = async () => {
    try {
      const newTab = await invoke<Tab>('create_tab');
      setTabs(prevTabs => [
        ...prevTabs.map(tab => ({ ...tab, isActive: false })),
        { ...newTab, isActive: true, canGoBack: false, canGoForward: false }
      ]);
    } catch (error) {
      console.error('Failed to create tab:', error);
    }
  };

  const handleCloseTab = async (tabId: string) => {
    try {
      await invoke('close_tab', { tabId });
      setTabs(prevTabs => {
        const updatedTabs = prevTabs.filter(tab => tab.id !== tabId);
        if (updatedTabs.length > 0) {
          const lastTab = updatedTabs[updatedTabs.length - 1];
          return updatedTabs.map(tab => ({
            ...tab,
            isActive: tab.id === lastTab.id
          }));
        }
        return updatedTabs;
      });
    } catch (error) {
      console.error('Failed to close tab:', error);
    }
  };

  const handleNavigate = async (url: string) => {
    if (!activeTab) return;
    
    try {
      setIsLoading(true);
      const processedUrl = url.startsWith('http') ? url : `https://${url}`;
      await invoke('navigate_to', { 
        url: processedUrl,
        tabId: activeTab.id 
      });
      setCurrentUrl(processedUrl);
    } catch (error) {
      console.error('Navigation failed:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (currentUrl) {
      handleNavigate(currentUrl);
    }
  };

  const handleTabClick = (tab: Tab) => {
    setTabs(prevTabs => 
      prevTabs.map(t => ({ ...t, isActive: t.id === tab.id }))
    );
    setActiveTab(tab);
    setCurrentUrl(tab.url);
  };

  const handleSearch = (input: string) => {
    if (input.includes('.') && !input.includes(' ')) {
      handleNavigate(input);
    } else {
      handleNavigate(`https://www.google.com/search?q=${encodeURIComponent(input)}`);
    }
  };

  useEffect(() => {
    const active = tabs.find(tab => tab.isActive);
    setActiveTab(active || null);
    if (active) {
      setCurrentUrl(active.url);
    }
  }, [tabs]);

  return (
    <div className="browser-container">
      <div className="browser-tabs">
        {tabs.map(tab => (
          <div 
            key={tab.id} 
            className={`tab ${tab.isActive ? 'active' : ''}`}
            onClick={() => handleTabClick(tab)}
          >
            <span className="tab-title">{tab.title || 'New Tab'}</span>
            <button 
              className="close-tab"
              onClick={(e) => {
                e.stopPropagation();
                handleCloseTab(tab.id);
              }}
            >
              ×
            </button>
          </div>
        ))}
        <button className="new-tab" onClick={handleNewTab}>+</button>
      </div>
      
      <div className="browser-content">
        <div className="webview-container">
          {isLoading && (
            <div className="loading-indicator">
              <div className="spinner"></div>
            </div>
          )}
          <webview
            id={`webview-${activeTab?.id}`}
            src={activeTab?.url || 'about:blank'}
            style={{ width: '100%', height: '100%' }}
          />
        </div>
      </div>
      
      <div className="browser-toolbar">
        <div className="nav-buttons">
          <button 
            className="nav-button" 
            onClick={() => activeTab && invoke('go_back', { tabId: activeTab.id })}
            disabled={!activeTab?.canGoBack}
          >
            <ArrowLeft size={20} />
          </button>
          <button 
            className="nav-button" 
            onClick={() => activeTab && invoke('go_forward', { tabId: activeTab.id })}
            disabled={!activeTab?.canGoForward}
          >
            <ArrowRight size={20} />
          </button>
          <button 
            className="nav-button" 
            onClick={() => activeTab && handleNavigate(activeTab.url)}
          >
            <RotateCcw size={20} />
          </button>
          <button 
            className="nav-button" 
            onClick={() => handleNavigate('about:blank')}
          >
            <Home size={20} />
          </button>
        </div>
        <form onSubmit={handleSubmit} className="url-form">
          <input
            type="text"
            className="url-input"
            value={currentUrl}
            onChange={(e) => setCurrentUrl(e.target.value)}
            placeholder="Enter URL or search terms"
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                handleSearch(currentUrl);
              }
            }}
          />
        </form>
      </div>
    </div>
  );
}

export default App;