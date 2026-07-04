import './EverTalkApp.css';
import { useEverTalkController } from '../hooks';
import { ChatStage } from './ChatStage';
import { SettingsPanel } from './SettingsPanel';
import { SpiritProfilePanel } from './SpiritProfilePanel';
import { SpiritRoster } from './SpiritRoster';
export function EverTalkApp() {
    const controller = useEverTalkController();
    if (controller.appInitializing) {
        return (<div className="ever-loading">
        <div className="ever-loading__ring"/>
        <strong>에버톡 로컬 데이터베이스 연결 중</strong>
      </div>);
    }
    return (<div className={`ever-app-shell ${controller.profileCollapsed ? 'is-profile-collapsed' : ''} ${controller.rosterCollapsed ? 'is-roster-collapsed' : ''}`}>
      <SpiritRoster spirits={controller.filteredSpirits} activeSpiritId={controller.activeSpiritId} defaultPersonaId={controller.defaultPersonaId} searchQuery={controller.searchQuery} loadError={controller.personaLoadError} activeTab={controller.activeRosterTab} collapsed={controller.rosterCollapsed} onSearchChange={controller.setSearchQuery} onSelect={controller.selectSpirit} onSetDefault={controller.setDefaultSpirit} onTabChange={controller.setActiveRosterTab} onToggleCollapsed={() => controller.setRosterCollapsed(!controller.rosterCollapsed)}/>
      <ChatStage activeDetail={controller.activeDetail} activeStageTab={controller.activeStageTab} activeRoom={controller.activeRoom} llmStatus={controller.llmStatus} messages={controller.messages} inputText={controller.inputText} isTyping={controller.isTyping} onInputChange={controller.setInputText} onSendMessage={controller.sendMessage} onStageTabChange={controller.setActiveStageTab} messageEndRef={controller.messageEndRef}/>
      <SpiritProfilePanel activeDetail={controller.activeDetail} collapsed={controller.profileCollapsed} systemStatuses={controller.systemStatuses} styles={controller.styles} activeStyle={controller.activeStyle} isSyncing={controller.isSyncing} onSyncStyles={controller.syncStyles} onSelectStyle={controller.selectStyle} onToggleCollapsed={() => controller.setProfileCollapsed(!controller.profileCollapsed)} onOpenSettings={controller.openSettings} isTraining={controller.isTraining} trainingSummary={controller.trainingSummary} trainingError={controller.trainingError} onTrainPersona={controller.trainPersona}/>
      <SettingsPanel open={controller.settingsOpen} settings={controller.appSettings} isResetting={controller.isResetting} resetSummary={controller.resetSummary} resetError={controller.resetError} onClose={controller.closeSettings} onReset={controller.resetAppData}/>
    </div>);
}
