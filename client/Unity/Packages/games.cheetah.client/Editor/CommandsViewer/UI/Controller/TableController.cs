using Cheetah.Matches.Realtime.GRPC.Admin;
using Games.Cheetah.Client.Editor.CommandsViewer.Provider;
using Games.Cheetah.Client.Editor.UIElements.Table;

namespace Games.Cheetah.Client.Editor.CommandsViewer.UI.Controller
{
    public class TableController
    {
        private readonly TableElement table;
        private readonly TracedCommandsProvider provider;
        private readonly Columns columns;

        public TableController(TableElement table,
            Columns columns,
            TracedCommandsProvider provider)
        {
            this.provider = provider;
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
                table.AddColumn(column.header, column.minWidth, column.maxWidth, column.flexGrow, item => column.GetValue((Command)item));
            }

            table.SetData(provider.GetCommands());
        }


        public void Update()
        {
            table.Update();
        }
    }
}