import './EverTalkApp.css';

import { useEverTalkController } from '../hooks';
import { ChatStage } from './ChatStage';
import { SpiritProfilePanel } from './SpiritProfilePanel';
import { SpiritRoster } from './SpiritRoster';

export function EverTalkApp() {
  const controller = useEverTalkController();

  if (controller.appInitializing) {
    return (
      <div className="ever-loading">
        <div className="ever-loading__ring" />
        <strong>에버톡 로컬 데이터베이스 연결 중</strong>
      </div>
    );
  }

  return (
    <div className="ever-app-shell">
      <SpiritRoster
        spirits={controller.filteredSpirits}
        activeSpiritId={controller.activeSpiritId}
        searchQuery={controller.searchQuery}
        onSearchChange={controller.setSearchQuery}
        onSelect={controller.selectSpirit}
      />
      <ChatStage
        activeDetail={controller.activeDetail}
        activeRoom={controller.activeRoom}
        llmStatus={controller.llmStatus}
        messages={controller.messages}
        inputText={controller.inputText}
        isTyping={controller.isTyping}
        onInputChange={controller.setInputText}
        onSendMessage={controller.sendMessage}
        messageEndRef={controller.messageEndRef}
      />
      <SpiritProfilePanel
        activeDetail={controller.activeDetail}
        styles={controller.styles}
        activeStyle={controller.activeStyle}
        isSyncing={controller.isSyncing}
        onSyncStyles={controller.syncStyles}
        onSelectStyle={controller.selectStyle}
      />
    </div>
  );
}
