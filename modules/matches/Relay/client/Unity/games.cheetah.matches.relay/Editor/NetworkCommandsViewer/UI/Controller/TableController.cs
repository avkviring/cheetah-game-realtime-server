using Cheetah.Matches.Factory.Editor.Configurations;
using Cheetah.Matches.Relay.Editor.GRPC;
using Cheetah.Matches.Relay.Editor.NetworkCommandsViewer.Provider;
using Cheetah.Matches.Relay.Editor.UIElements.Table;

namespace Cheetah.Matches.Relay.Editor.NetworkCommandsViewer.UI.Controller
{
    public class TableController
    {
        private readonly TableElement table;
        private readonly TracedCommandsProvider provider;
        private readonly ConfigurationsProvider configurationsProvider;
        private readonly Columns columns;

        public TableController(TableElement table,
            Columns columns,
            TracedCommandsProvider provider, ConfigurationsProvider configurationsProvider)
        {
            this.provider = provider;
            this.configurationsProvider = configurationsProvider;
            this.columns = columns;
            this.table = table;
            table.EnableAutoScroll();
            columns.OnActiveColumnsUpdate += ConfigureTable;
            ConfigureTable();
        }

        private void ConfigureTable()
        {
            table.Reset();
            foreach (var column in columns.GetEnabledColumns())
            {
                table.AddColumn(column.header, column.minWidth, column.maxWidth, column.flexGrow, item => column.GetValue((Command)item, configurationsProvider));
            }
            table.SetData(provider.GetCommands());
        }


        public void Update()
        {
            table.Update();
        }
    }
}